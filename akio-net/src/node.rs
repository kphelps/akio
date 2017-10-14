use super::ipc::*;
use super::rpc;
use super::client::create_client;
use super::client_state::*;
use super::server::{start_server, Server};
use super::errors::*;
use futures::future;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio_core::reactor::{Handle};

#[derive(Clone)]
pub struct RemoteNode {
    inner: Arc<RemoteNodeInner>,
}

impl RemoteNode {
    pub fn new(handle: &Handle, listen_address: &SocketAddr) -> Result<Self> {
        let (client_event_sender, client_event_receiver) = mpsc::channel(100);
        let my_id = new_client_id();
        let server = start_server(my_id, &handle, listen_address, client_event_sender.clone())?;
        let inner = RemoteNodeInner::new(
            server,
            client_event_sender,
            handle
        );
        let node = RemoteNode {
            inner: Arc::new(inner),
        };
        node.clone().start_event_handler(handle, client_event_receiver);
        Ok(node)
    }

    pub fn connect(&self, addr: &SocketAddr) {
        self.inner.connect_tx(addr)
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
            ClientEvent::TxConnecting(client_id) => self.inner.client_tx_connecting(client_id),
            ClientEvent::TxConnected(client_id, state) => self.inner.client_tx_connected(client_id, state),
            ClientEvent::TxDisconnected(client_id) => self.inner.client_tx_disconnected(client_id),
            ClientEvent::RequestReceived(client_id, message) => self.inner.handle_request(client_id, message),
            ClientEvent::ResponseReceived(client_id, message) => self.inner.handle_response(client_id, message),
        }
    }
}

enum ConnectionState<T> {
    Disconnected,
    Connecting,
    Connected(T)
}

impl<T> ConnectionState<T> {
    pub fn is_disconnected(&self) -> bool {
        match *self {
            ConnectionState::Disconnected => true,
            _ => false
        }
    }

    pub fn state(&self) -> Option<&T> {
        match *self {
            ConnectionState::Connected(ref inner) => Some(inner),
            _ => None
        }
    }
}

struct RemoteConnection {
    rx: ConnectionState<ClientRxState>,
    tx: ConnectionState<ClientTxState>,
}

impl RemoteConnection {
    pub fn new() -> Self {
        Self {
            rx: ConnectionState::Disconnected,
            tx: ConnectionState::Disconnected,
        }
    }

    pub fn respond(&self, id: u64, result: Result<()>) -> Option<impl Future<Item = (), Error = ()>> {
        self.rx.state().map(|client| client.respond(id, result))
    }

    pub fn request(&self, payload: rpc::Request_oneof_payload) -> Option<impl Future<Item = (), Error = ()>> {
        self.tx.state().map(|client| client.request(payload))
    }

    pub fn is_disconnected(&self) -> bool {
        self.rx.is_disconnected() && self.tx.is_disconnected()
    }
}

struct RemoteNodeInner {
    server: Server,
    client_states: Mutex<HashMap<ClientId, RemoteConnection>>,
    client_event_sender: mpsc::Sender<ClientEvent>,
    handle: Handle,
}

impl RemoteNodeInner {
    pub fn new(server: Server, client_event_sender: mpsc::Sender<ClientEvent>, handle: &Handle) -> Self {
        Self {
            server: server,
            client_states: Mutex::new(HashMap::new()),
            client_event_sender: client_event_sender,
            handle: handle.clone(),
        }
    }

    pub fn id(&self) -> ClientId {
        self.server.id.clone()
    }

    pub fn connect_tx(&self, addr: &SocketAddr) {
        create_client(
            self.id(),
            &self.server.listen_addr,
            &self.handle,
            addr,
            self.client_event_sender.clone(),
        )
    }

    pub fn client_rx_connected(&self, client_id: ClientId, state: ClientRxState) {
        println!("[{}] rx connnected", client_id);
        let addr = state.addr();
        self.with_client(client_id, |client| client.rx = ConnectionState::Connected(state));
        if self.with_client(client_id, |client| client.tx.is_disconnected()) {
            self.connect_tx(&addr);
        }
    }

    pub fn client_rx_disconnected(&self, client_id: ClientId) {
        println!("[{}] rx disconnnected", client_id);
        self.with_client(client_id, |client| client.rx = ConnectionState::Disconnected)
    }

    pub fn client_tx_connecting(&self, client_id: ClientId) {
        println!("[{}] tx connecting...", client_id);
        self.with_client(client_id, |client| client.tx = ConnectionState::Connecting)
    }

    pub fn client_tx_connected(&self, client_id: ClientId, state: ClientTxState) {
        println!("[{}] tx connnected", client_id);
        self.with_client(client_id, |client| client.tx = ConnectionState::Connected(state));
    }

    pub fn client_tx_disconnected(&self, client_id: ClientId) {
        println!("[{}] tx disconnnected", client_id);
        self.with_client(client_id, |client| client.tx = ConnectionState::Disconnected)
    }

    pub fn with_client<F, R>(&self, client_id: ClientId, f: F) -> R
        where F: FnOnce(&mut RemoteConnection) -> R
    {
        let mut locked = self.client_states.lock();
        let mut client = locked
            .entry(client_id)
            .or_insert_with(RemoteConnection::new);
        f(&mut client)
    }

    pub fn handle_request(&self, client_id: ClientId, request: rpc::Request) {
        if request.payload.is_none() {
            return
        }
        println!("[{}] received: {:?}", client_id, request);
        match request.payload.unwrap() {
            _ => ()
        };
    }

    pub fn handle_response(&self, client_id: ClientId, response: rpc::Response) {

    }

    fn client_is_disconnected(&self, client_id: ClientId) -> bool {
        self.with_client(client_id, |conn| conn.is_disconnected())
    }
}
