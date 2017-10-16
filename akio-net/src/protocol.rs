use super::client_state::*;
use super::errors::*;
use super::ipc::*;
use super::rpc;
use bytes::BytesMut;
use futures::prelude::*;
use futures::sync::mpsc;
use protobuf;
use std::net::SocketAddr;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::length_delimited;
use uuid::Uuid;

fn bind_transport<T: AsyncRead + AsyncWrite>(
    io: T,
) -> (
    impl Sink<SinkItem = BytesMut, SinkError = Error>,
    impl Stream<Item = BytesMut, Error = Error>,
) {
    length_delimited::Framed::new(io)
        .sink_map_err(|e| e.into())
        .map_err(|e| e.into())
        .split()
}

fn bind_client<C: ClientState, T: AsyncRead + AsyncWrite>(
    io: T,
) -> (
    impl Sink<SinkItem = C::Send, SinkError = ()>,
    impl Stream<Item = C::Receive, Error = Error>,
) {
    let (raw_sender, raw_receiver) = bind_transport(io);
    let sender = raw_sender
        .with(|msg| serialize_message(&msg))
        .sink_map_err(|_| ());
    let receiver = raw_receiver.and_then(parse_message::<C::Receive>);
    (sender, receiver)
}

fn parse_message<T>(bytes: BytesMut) -> Result<T>
where
    T: protobuf::MessageStatic,
{
    Ok(protobuf::parse_from_carllerche_bytes::<T>(&bytes.freeze())?)
}

fn serialize_message<T>(message: &T) -> Result<BytesMut>
where
    T: protobuf::MessageStatic,
{
    message
        .write_to_bytes()
        .map(|x| x.into())
        .map_err(|e| e.into())
}

fn do_handshake<S, R>(
    client_id: ClientId,
    address: &SocketAddr,
    sender: S,
    receiver: R,
) -> impl Future<Item = (ClientId, S, R), Error = ()>
where
    S: Sink<SinkItem = rpc::Request, SinkError = ()>,
    R: Stream<Item = rpc::Response, Error = Error>,
{
    let mut handshake = rpc::StartHandshake::new();
    handshake.set_client_id(client_id.as_bytes().as_ref().into());
    handshake.set_address(format!("{}", address.ip()).into());
    handshake.set_port(u32::from(address.port()));
    let payload = rpc::Request_oneof_payload::start_handshake(handshake);
    let request = ClientTxState::make_request(payload);
    sender
        .send(request)
        .and_then(|sender2| {
            receiver.into_future().map(|x| (sender2, x)).map_err(|_| ())
        })
        .and_then(|(sender2, (response, receiver2))| {
            response.ok_or(()).and_then(|inner| {
                if inner.error == rpc::ErrorCode::NONE && inner.has_finish_handshake() {
                    let uuid =
                        Uuid::from_bytes(&inner.get_finish_handshake().client_id).map_err(|_| ())?;
                    Ok((uuid, sender2, receiver2))
                } else {
                    Err(())
                }
            })
        })
}

fn receive_handshake<S, R>(
    my_id: ClientId,
    sender: S,
    receiver: R,
) -> impl Future<Item = (rpc::StartHandshake, S, R), Error = ()>
where
    S: Sink<SinkItem = rpc::Response, SinkError = ()>,
    R: Stream<Item = rpc::Request, Error = Error>,
{
    receiver
        .into_future()
        .map_err(|_| ())
        .and_then(|(request, receiver)| {
            request.ok_or(()).map(|request| (request, receiver))
        })
        .and_then(move |(mut request, receiver)| {
            let mut handshake = rpc::FinishHandshake::new();
            handshake.set_client_id(my_id.as_bytes().as_ref().into());
            let payload = rpc::Response_oneof_payload::finish_handshake(handshake);
            let response = ClientRxState::make_response_with_payload(&request, payload);
            sender.send(response).map(move |sender| {
                (request.take_start_handshake(), sender, receiver)
            })
        })
}

fn when_connected<T, S, R>(
    params: T::Params,
    sender: S,
    receiver: R,
    handle: &Handle,
    client_event_sender: mpsc::Sender<ClientEvent>,
) -> T
where
    T: ClientState + Clone + 'static,
    S: Sink<SinkItem = T::Send, SinkError = ()> + 'static,
    R: Stream<Item = T::Receive, Error = Error> + 'static,
{
    let (from_socket, to_socket) = mpsc::channel(20);
    let client = T::new(params, from_socket);
    let connect_event_f = client_event_sender
        .clone()
        .send(client.connected_event())
        .map(|_| ())
        .map_err(|_| ());
    handle.spawn(connect_event_f);

    let client_id = client.id();
    let recv_client = client.clone();
    let from_socket_stream = receiver
        .map(move |message| recv_client.on_receive(message))
        .map_err(move |err| error!("[{}] Recv error: {}", client_id, err));
    let socket_read_stream = client_event_sender
        .clone()
        .sink_map_err(|_| ())
        .send_all(from_socket_stream);
    let send_stream = sender.send_all(to_socket);
    let disconnect_message = client.disconnected_event();
    let connection = send_stream
        .map(|_| ())
        .select(socket_read_stream.map(|_| ()))
        .then(move |_| {
            client_event_sender
                .clone()
                .send(disconnect_message)
                .map(|_| ())
                .map_err(|_| ())
        });

    handle.spawn(connection);
    client
}

pub(crate) fn initialize_rx_stream(
    my_id: ClientId,
    stream: TcpStream,
    handle: &Handle,
    client_event_sender: mpsc::Sender<ClientEvent>,
) -> impl Future<Item = ClientRxState, Error = ()> {
    let (sender, receiver) = bind_client::<ClientRxState, TcpStream>(stream);
    let handle = handle.clone();
    receive_handshake(my_id, sender, receiver).map(move |(handshake, sender, receiver)| {
        when_connected(handshake, sender, receiver, &handle, client_event_sender)
    })
}

pub(crate) fn initialize_tx_stream(
    my_id: ClientId,
    listen_addr: &SocketAddr,
    stream: TcpStream,
    handle: Handle,
    client_event_sender: mpsc::Sender<ClientEvent>,
) -> impl Future<Item = ClientTxState, Error = ()> {
    let listen_addr = *listen_addr;
    let (sender, receiver) = bind_client::<ClientTxState, TcpStream>(stream);
    do_handshake(my_id, &listen_addr, sender, receiver).map(
        move |(client_id, sender2, receiver2)| {
            when_connected(client_id, sender2, receiver2, &handle, client_event_sender)
        },
    )
}
