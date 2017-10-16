use super::errors::*;
use super::ipc::*;
use super::rpc;
use futures::prelude::*;
use futures::sync::mpsc;
use protobuf;
use rand;
use rand::Rng;
use std::net::SocketAddr;
use std::rc::Rc;
use uuid::Uuid;

pub(crate) trait ClientState {
    type Receive: protobuf::MessageStatic;
    type Send: protobuf::MessageStatic;
    type Params;

    fn new(params: Self::Params, sender: mpsc::Sender<Self::Send>) -> Self;

    fn id(&self) -> ClientId;

    fn on_receive(&self, message: Self::Receive) -> ClientEvent;

    fn connected_event(&self) -> ClientEvent;

    fn disconnected_event(&self) -> ClientEvent;
}

#[derive(Debug)]
struct ClientRxStateInner {
    id: ClientId,
    addr: SocketAddr,
    tx: mpsc::Sender<rpc::Response>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClientRxState {
    inner: Rc<ClientRxStateInner>,
}

impl ClientState for ClientRxState {
    type Receive = rpc::Request;
    type Send = rpc::Response;
    type Params = rpc::StartHandshake;

    fn new(params: Self::Params, tx: mpsc::Sender<Self::Send>) -> Self {
        let inner = ClientRxStateInner {
            // TODO unwrap is bad, don't be an idiot
            id: Uuid::from_bytes(&params.client_id).unwrap(),
            addr: format!("{}:{}", params.address, params.port)
                .parse()
                .unwrap(),
            tx: tx,
        };
        Self {
            inner: Rc::new(inner),
        }
    }

    fn id(&self) -> ClientId {
        self.inner.id
    }

    fn on_receive(&self, message: Self::Receive) -> ClientEvent {
        ClientEvent::RequestReceived(self.id(), message)
    }

    fn connected_event(&self) -> ClientEvent {
        ClientEvent::RxConnected(self.id(), self.clone())
    }

    fn disconnected_event(&self) -> ClientEvent {
        ClientEvent::RxDisconnected(self.id())
    }
}

impl ClientRxState {
    pub fn addr(&self) -> SocketAddr {
        self.inner.addr
    }

    pub fn make_response_to_parts(id: u64, _result: Result<()>) -> rpc::Response {
        let mut response = rpc::Response::new();
        response.set_id(id);
        response
    }

    pub fn make_response(request: &rpc::Request) -> rpc::Response {
        Self::make_response_to_parts(request.id, Ok(()))
    }

    pub fn make_response_with_payload(
        request: &rpc::Request,
        payload: rpc::Response_oneof_payload,
    ) -> rpc::Response {
        let mut response = Self::make_response(request);
        response.payload = Some(payload);
        response
    }

    pub fn respond(&self, id: u64, result: Result<()>) -> impl Future<Item = (), Error = ()> {
        let response = Self::make_response_to_parts(id, result);
        self.inner
            .tx
            .clone()
            .send(response)
            .map(|_| ())
            .map_err(|_| ())
    }
}


#[derive(Debug)]
struct ClientTxStateInner {
    id: ClientId,
    tx: mpsc::Sender<rpc::Request>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClientTxState {
    inner: Rc<ClientTxStateInner>,
}

impl ClientState for ClientTxState {
    type Receive = rpc::Response;
    type Send = rpc::Request;
    type Params = ClientId;

    fn new(params: Self::Params, tx: mpsc::Sender<Self::Send>) -> Self {
        let inner = ClientTxStateInner {
            id: params,
            tx: tx,
        };
        Self {
            inner: Rc::new(inner),
        }
    }

    fn id(&self) -> ClientId {
        self.inner.id
    }

    fn on_receive(&self, message: Self::Receive) -> ClientEvent {
        ClientEvent::ResponseReceived(self.id(), message)
    }

    fn connected_event(&self) -> ClientEvent {
        ClientEvent::TxConnected(self.id(), self.clone())
    }

    fn disconnected_event(&self) -> ClientEvent {
        ClientEvent::TxDisconnected(self.id())
    }
}

impl ClientTxState {
    pub fn make_request(payload: rpc::Request_oneof_payload) -> rpc::Request {
        Self::make_request_raw(Some(payload))
    }

    pub fn make_request_raw(payload: Option<rpc::Request_oneof_payload>) -> rpc::Request {
        let mut request = rpc::Request::new();
        request.set_id(rand::thread_rng().next_u64());
        request.payload = payload;
        request
    }

    pub fn request(
        &self,
        payload: rpc::Request_oneof_payload,
    ) -> (u64, impl Future<Item = (), Error = ()>) {
        self.request_raw(Some(payload))
    }

    pub fn request_raw(
        &self,
        payload: Option<rpc::Request_oneof_payload>,
    ) -> (u64, impl Future<Item = (), Error = ()>) {
        let request = Self::make_request_raw(payload);
        (
            request.id,
            self.inner
                .tx
                .clone()
                .send(request)
                .map(|_| ())
                .map_err(|_| ()),
        )
    }
}
