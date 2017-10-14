use super::client_state::{ClientRxState, ClientTxState};
use super::rpc;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) enum ClientMessage {
    Request(rpc::Request),
    Response(rpc::Response),
}

pub(crate) type ClientId = Uuid;

pub fn new_client_id() -> ClientId {
    Uuid::new_v4()
}

#[derive(Debug)]
pub(crate) enum ClientEvent {
    RxConnected(ClientId, ClientRxState),
    RxDisconnected(ClientId),
    TxConnecting(ClientId),
    TxConnected(ClientId, ClientTxState),
    TxDisconnected(ClientId),
    RequestReceived(ClientId, rpc::Request),
    ResponseReceived(ClientId, rpc::Response),
}
