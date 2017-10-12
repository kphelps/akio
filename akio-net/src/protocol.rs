use super::client_state::ClientState;
use super::ipc::*;
use bytes::BytesMut;
use futures::prelude::*;
use futures::sync::mpsc;
use tokio_core::reactor::Handle;
use tokio_core::net::TcpStream;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::length_delimited;

fn bind_transport<T: AsyncRead + AsyncWrite>(io: T)
    -> length_delimited::Framed<T>
{
    length_delimited::Framed::new(io)
}

fn parse_message(_bytes: BytesMut) -> ClientMessage {
    ClientMessage {

    }
}

fn serialize_message(_message: ClientMessage) -> BytesMut {
    BytesMut::new()
}

pub(crate) fn initialize_stream<T>(
    client_id: ClientId,
    stream: TcpStream,
    handle: &Handle,
    client_event_sender: mpsc::Sender<ClientEvent>
) -> T
    where T: ClientState
{
    let framed = bind_transport(stream);
    let (raw_sender, raw_receiver) = framed.split();
    let (from_socket, to_socket) = mpsc::channel(20);

    let client = T::new(client_id, from_socket);

    let connect_event_f = client_event_sender.clone()
        .send(client.connected_event())
        .map(|_| ())
        .map_err(|_| ());
    handle.spawn(connect_event_f);

    let client_id = client.id();
    let from_socket_stream = raw_receiver
        .map(parse_message)
        .map(move |message| ClientEvent::MessageReceived(client_id, message))
        .map_err(|_| ());
    let socket_read_stream = client_event_sender.clone().sink_map_err(|_| ()).send_all(from_socket_stream);
    handle.spawn(socket_read_stream.map(|_| ()));

    let sender = raw_sender.sink_map_err(|_| ()).send_all(to_socket.map(serialize_message));
    handle.spawn(sender.map(|_| ()));
    // TODO select both futures + handle disconnect

    client
}
