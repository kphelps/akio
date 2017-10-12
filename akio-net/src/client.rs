use super::client_state::ClientTxState;
use super::errors::*;
use super::ipc::*;
use super::protocol::initialize_stream;
use futures::prelude::*;
use futures::sync::mpsc;
use std::net::SocketAddr;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;

pub(crate) fn create_client(
    server_id: ClientId,
    in_handle: &Handle,
    target: &SocketAddr,
    client_event_sender: mpsc::Sender<ClientEvent>
) -> Result<()> {
    let handle = in_handle.clone();
    let client_stream = TcpStream::connect(target, in_handle).and_then(move |stream| {
        initialize_stream::<ClientTxState>(
            server_id,
            stream,
            &handle,
            client_event_sender.clone()
        );
        Ok(())
    }).map_err(|_| ());
    in_handle.spawn(client_stream);
    Ok(())
}
