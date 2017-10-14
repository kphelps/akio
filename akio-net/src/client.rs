use super::client_state::ClientTxState;
use super::errors::*;
use super::ipc::*;
use super::protocol::initialize_tx_stream;
use futures::prelude::*;
use futures::sync::mpsc;
use std::net::SocketAddr;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;

pub(crate) fn create_client(
    server_id: ClientId,
    in_listener_addr: &SocketAddr,
    in_handle: &Handle,
    target: &SocketAddr,
    client_event_sender: mpsc::Sender<ClientEvent>
) {
    let handle = in_handle.clone();
    let listener_addr = in_listener_addr.clone();
    let client_stream = TcpStream::connect(target, in_handle)
        .map_err(|_| ())
        .and_then(move |stream| {
            initialize_tx_stream(
                server_id,
                &listener_addr,
                stream,
                handle,
                client_event_sender.clone()
            )
        }).map(|_| ());
    in_handle.spawn(client_stream);
}
