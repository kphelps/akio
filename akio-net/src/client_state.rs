use super::ipc::*;
use std::rc::Rc;
use futures::sync::mpsc;

pub(crate) trait ClientState {
    fn new(id: ClientId, sender: mpsc::Sender<ClientMessage>) -> Self;

    fn id(&self) -> ClientId;

    fn connected_event(&self) -> ClientEvent;

    fn disconnected_event(&self) -> ClientEvent;
}

#[derive(Debug)]
struct ClientRxStateInner {
    id: ClientId,
    tx: mpsc::Sender<ClientMessage>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClientRxState {
    inner: Rc<ClientRxStateInner>
}

impl ClientState for ClientRxState {
    fn new(id: ClientId, tx: mpsc::Sender<ClientMessage>) -> Self {
        let inner = ClientRxStateInner {
            id: id,
            tx: tx,
        };
        Self {
            inner: Rc::new(inner)
        }
    }

    fn id(&self) -> ClientId {
        self.inner.id.clone()
    }

    fn connected_event(&self) -> ClientEvent {
        ClientEvent::RxConnected(self.id(), self.clone())
    }

    fn disconnected_event(&self) -> ClientEvent {
        ClientEvent::RxDisconnected(self.id())
    }
}




#[derive(Debug)]
struct ClientTxStateInner {
    id: ClientId,
    tx: mpsc::Sender<ClientMessage>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClientTxState {
    inner: Rc<ClientTxStateInner>
}

impl ClientState for ClientTxState {
    fn new(id: ClientId, tx: mpsc::Sender<ClientMessage>) -> Self {
        let inner = ClientTxStateInner {
            id: id,
            tx: tx,
        };
        Self {
            inner: Rc::new(inner)
        }
    }

    fn id(&self) -> ClientId {
        self.inner.id.clone()
    }

    fn connected_event(&self) -> ClientEvent {
        ClientEvent::TxConnected(self.id(), self.clone())
    }

    fn disconnected_event(&self) -> ClientEvent {
        ClientEvent::TxDisconnected(self.id())
    }
}

