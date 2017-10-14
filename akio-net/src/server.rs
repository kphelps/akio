use super::errors::*;
use super::ipc::*;
use super::protocol::initialize_rx_stream;
use futures::prelude::*;
use futures::sync::mpsc;
use std::net::SocketAddr;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Handle;

pub(crate) struct Server {
    pub id: ClientId,
    pub listen_addr: SocketAddr,
}

pub(crate) fn start_server(
    my_id: ClientId,
    in_handle: &Handle,
    listen_addr: &SocketAddr,
    client_event_sender: mpsc::Sender<ClientEvent>
) -> Result<Server> {
    let listener = TcpListener::bind(listen_addr, in_handle)?;
    let handle = in_handle.clone();
    let server_id = my_id.clone();
    let listener_stream = listener.incoming()
        .map_err(|_| ())
        .for_each(move |(stream, _client_addr)| {
            initialize_rx_stream(
                my_id,
                stream,
                &handle,
                client_event_sender.clone()
            ).map(|_| ())
        });
    in_handle.spawn(listener_stream);
    Ok(Server{
        id: server_id,
        listen_addr: listen_addr.clone(),
    })
}
