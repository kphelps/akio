use super::ipc::*;
use super::rpc;
use super::client::create_client;
use super::client_state::*;
use super::server::{start_server, Server};
use super::errors::*;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use tokio_core::reactor::{Handle};
use tokio_timer;

#[derive(Clone)]
pub struct RemoteNode {
    inner: Arc<RemoteNodeInner>,
}

impl RemoteNode {
    pub fn new(handle: &Handle, listen_address: &SocketAddr) -> Result<Self> {
        let (client_event_sender, client_event_receiver) = mpsc::channel(100);
        let my_id = new_client_id();
        let server = start_server(my_id, &handle, listen_address, client_event_sender.clone())?;
        info!("Listening on '{}'", listen_address);
        let inner = RemoteNodeInner::new(
            server,
            client_event_sender,
            handle
        );
        let node = RemoteNode {
            inner: Arc::new(inner),
        };
        node.start_hearbeat_task();
        node.clone().start_event_handler(client_event_receiver);
        Ok(node)
    }

    pub fn connect(&self, addr: &SocketAddr) {
        self.inner.connect_tx(addr)
    }

    fn start_event_handler(self, server_event_stream: mpsc::Receiver<ClientEvent>) {
        let handle = self.inner.handle.clone();
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
            ClientEvent::RequestReceived(client_id, message) => self.inner.handle_request(client_id, message),
            ClientEvent::ResponseReceived(client_id, message) => self.inner.handle_response(client_id, message),
        }
    }

    fn start_hearbeat_task(&self) {
        let clients = self.inner.clone();
        let ticker = tokio_timer::wheel()
            .build()
            .interval(Duration::from_secs(3))
            .for_each(move |_| Ok(clients.send_heartbeats()))
            .map_err(|_| ());
        self.inner.handle.spawn(ticker);
    }
}

enum ConnectionState<T> {
    Disconnected,
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
    client_id: ClientId,
    rx: ConnectionState<ClientRxState>,
    tx: ConnectionState<ClientTxState>,
    handle: Handle,
}

impl RemoteConnection {
    pub fn new(client_id: ClientId, handle: &Handle) -> Self {
        Self {
            client_id: client_id,
            rx: ConnectionState::Disconnected,
            tx: ConnectionState::Disconnected,
            handle: handle.clone(),
        }
    }

    pub fn respond_ok(&self, id: u64) {
        self.respond(id, Ok(()))
    }

    pub fn respond(&self, id: u64, result: Result<()>) {
        let maybe_f = self.rx.state().map(|client| client.respond(id, result));
        match maybe_f {
            Some(f) => self.handle.spawn(f),
            None => error!("[{}] Failed to respond to '{}'", self.client_id, id),
        }
    }

    pub fn request(&self, payload: rpc::Request_oneof_payload) {
        self.request_raw(Some(payload))
    }

    pub fn request_raw(&self, payload: Option<rpc::Request_oneof_payload>) {
        let maybe_f = self.tx.state().map(|client| client.request_raw(payload));
        match maybe_f {
            Some(f) => self.handle.spawn(f),
            None => error!("[{}] Failed to send request", self.client_id),
        }
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
        info!("[{}] rx connnected", client_id);
        let addr = state.addr();
        self.with_client(client_id, |client| client.rx = ConnectionState::Connected(state));
        if self.with_client(client_id, |client| client.tx.is_disconnected()) {
            self.connect_tx(&addr);
        }
    }

    pub fn client_rx_disconnected(&self, client_id: ClientId) {
        info!("[{}] rx disconnnected", client_id);
        self.with_client(client_id, |client| client.rx = ConnectionState::Disconnected)
    }

    pub fn client_tx_connected(&self, client_id: ClientId, state: ClientTxState) {
        info!("[{}] tx connnected", client_id);
        self.with_client(client_id, |client| client.tx = ConnectionState::Connected(state));
    }

    pub fn client_tx_disconnected(&self, client_id: ClientId) {
        info!("[{}] tx disconnnected", client_id);
        self.with_client(client_id, |client| client.tx = ConnectionState::Disconnected)
    }

    pub fn with_client<F, R>(&self, client_id: ClientId, f: F) -> R
        where F: FnOnce(&mut RemoteConnection) -> R
    {
        let mut locked = self.client_states.lock();
        let mut client = locked
            .entry(client_id)
            .or_insert_with(|| RemoteConnection::new(client_id, &self.handle));
        f(&mut client)
    }

    pub fn handle_request(&self, client_id: ClientId, request: rpc::Request) {
        if request.payload.is_none() {
            self.with_client(client_id, |client| client.respond_ok(request.id));
        } else {
            debug!("[{}] received: {:?}", client_id, request);
            match request.payload.unwrap() {
                _ => ()
            };
        }
    }

    pub fn handle_response(&self, client_id: ClientId, response: rpc::Response) {
        debug!("[{}] response: {:?}", client_id, response)
    }

    pub fn send_heartbeats(&self) {
        let clients = self.client_states.lock();
        clients.iter().for_each(|(_, client)| {
            client.request_raw(None)
        });
    }
}
