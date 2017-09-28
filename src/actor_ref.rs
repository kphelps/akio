use super::ActorCellHandle;
use std::any::Any;

#[derive(Clone)]
pub struct ActorRef {
    handle: ActorCellHandle,
}

impl ActorRef {
    pub fn new(handle: ActorCellHandle) -> Self {
        Self { handle: handle }
    }

    pub fn send<T: Any + Send>(&self, message: T, sender: &ActorRef) {
        self.send_any(Box::new(message), sender)
    }

    pub fn send_any(&self, message: Box<Any + Send>, sender: &ActorRef) {
        self.handle.enqueue_message(message, sender.clone())
    }
}
