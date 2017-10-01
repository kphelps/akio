use super::ActorCell;
use futures::prelude::*;
use futures::sync::oneshot;
use std::any::Any;

#[derive(Clone)]
pub struct ActorRef {
    cell: ActorCell,
}

impl ActorRef {
    pub fn new(cell: ActorCell) -> Self {
        Self { cell: cell }
    }

    pub fn send<T: Any + Send>(&self, message: T, sender: &ActorRef) {
        self.send_any(Box::new(message), sender)
    }

    pub fn send_any(&self, message: Box<Any + Send>, sender: &ActorRef) {
        self.cell.enqueue_message(message, sender.clone())
    }

    pub fn ask<T>(&self,
                  message: Box<Any + Send>,
                  sender: &ActorRef)
                  -> Box<Future<Item = (), Error = ()>> {
        let (promise, f) = oneshot::channel();
        Box::new(f.map_err(|_| ()))
    }
}
