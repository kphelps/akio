use super::ActorCellHandle;
use futures::prelude::*;
use futures::sync::oneshot;
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

    pub fn ask<T>(&self,
                  message: Box<Any + Send>,
                  sender: &ActorRef)
                  -> Box<Future<Item = (), Error = ()>> {
        let (promise, f) = oneshot::channel();
        Box::new(f.map_err(|_| ()))
    }
}
