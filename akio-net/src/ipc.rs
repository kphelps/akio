use super::client_state::{ClientRxState, ClientTxState};
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct ClientMessage {

}

pub(crate) type ClientId = Uuid;

pub fn new_client_id() -> ClientId {
    Uuid::new_v4()
}

#[derive(Debug)]
pub(crate) enum ClientEvent {
    RxConnected(ClientId, ClientRxState),
    RxDisconnected(ClientId),
    TxConnected(ClientId, ClientTxState),
    TxDisconnected(ClientId),
    MessageReceived(ClientId, ClientMessage),
}
