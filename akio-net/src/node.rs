use super::ipc::*;
use super::client_state::*;
use super::server::{start_server, Server};
use super::errors::*;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio_core::reactor::Handle;

#[derive(Clone)]
pub struct RemoteNode {
    inner: Arc<RemoteNodeInner>,
}

impl RemoteNode {
    pub fn new(handle: &Handle, listen_port: u16) -> Result<Self> {
        let (client_event_sender, client_event_receiver) = mpsc::channel(100);
        let server = start_server(&handle, listen_port, client_event_sender)?;
        let inner = RemoteNodeInner::new(server);
        let node = RemoteNode {
            inner: Arc::new(inner),
        };
        node.clone().start_event_handler(handle, client_event_receiver);
        Ok(node)
    }

    fn start_event_handler(self, handle: &Handle, server_event_stream: mpsc::Receiver<ClientEvent>) {
        let event_handler_f = server_event_stream
            .for_each(move |event| Ok(self.handle_server_event(event)));
        handle.spawn(event_handler_f);
    }

    fn handle_server_event(&self, event: ClientEvent) {
        match event {
            ClientEvent::RxConnected(client_id, state) => self.inner.client_rx_connected(client_id, state),
            ClientEvent::RxDisconnected(client_id) => self.inner.client_rx_disconnected(client_id),
            ClientEvent::TxConnected(client_id, state) => self.inner.client_tx_connected(client_id, state),
            ClientEvent::TxDisconnected(client_id) => self.inner.client_tx_disconnected(client_id),
            ClientEvent::MessageReceived(client_id, message) => self.inner.handle_message(client_id, message)
        }
    }
}

struct ClientState {
    rx: Option<ClientRxState>,
    tx: Option<ClientTxState>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            rx: None,
            tx: None,
        }
    }
}

struct RemoteNodeInner {
    _server: Server,
    client_states: Mutex<HashMap<ClientId, ClientState>>,
}

impl RemoteNodeInner {
    pub fn new(server: Server) -> Self {
        Self {
            _server: server,
            client_states: Mutex::new(HashMap::new()),
        }
    }

    pub fn client_rx_connected(&self, client_id: ClientId, state: ClientRxState) {
        self.with_client(client_id, |client| client.rx = Some(state))
    }

    pub fn client_rx_disconnected(&self, client_id: ClientId) {
        self.with_client(client_id, |client| client.rx = None)
    }

    pub fn client_tx_connected(&self, client_id: ClientId, state: ClientTxState) {
        self.with_client(client_id, |client| client.tx = Some(state))
    }

    pub fn client_tx_disconnected(&self, client_id: ClientId) {
        self.with_client(client_id, |client| client.tx = None)
    }

    pub fn with_client<F, R>(&self, client_id: ClientId, f: F) -> R
        where F: FnOnce(&mut ClientState) -> R
    {
        let mut locked = self.client_states.lock();
        let mut client = locked
            .entry(client_id)
            .or_insert_with(ClientState::new);
        f(&mut client)
    }

    pub fn handle_message(&self, client_id: ClientId, message: ClientMessage) {
        println!("[{}] {:?}", client_id, message)
    }
}
