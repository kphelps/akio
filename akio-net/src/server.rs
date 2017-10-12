use super::client_state::ClientRxState;
use super::errors::*;
use super::ipc::*;
use super::protocol::initialize_stream;
use futures::prelude::*;
use futures::sync::mpsc;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Handle;

pub(crate) struct Server {

}

pub(crate) fn start_server(in_handle: &Handle, port: u16, client_event_sender: mpsc::Sender<ClientEvent>) -> Result<Server> {
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let listener = TcpListener::bind(&addr, in_handle)?;
    let handle = in_handle.clone();
    // TODO: Need to handshake id
    let listener_stream = listener.incoming().for_each(move |(stream, _client_addr)| {
        let client_id = new_client_id();
        initialize_stream::<ClientRxState>(
            client_id,
            stream,
            &handle,
            client_event_sender.clone()
        );
        Ok(())
    }).map_err(|_| ());
    in_handle.spawn(listener_stream);
    Ok(Server{})
}
