use super::{ActorCellHandle, AskActor, BaseActor};
use futures::prelude::*;
use futures::sync::oneshot;
use std::any::Any;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorRef {
    cell: ActorCellHandle,
}

impl ActorRef {
    pub fn new(cell: ActorCellHandle) -> Self {
        Self { cell: cell }
    }

    pub fn send<T: Any + Send>(&self, message: T, sender: &ActorRef) {
        self.send_any(Box::new(message), sender)
    }

    pub fn send_any(&self, message: Box<Any + Send>, sender: &ActorRef) {
        self.cell.enqueue_message(message, sender.clone())
    }

    pub fn spawn<A>(&mut self, id: Uuid, actor: A) -> ActorRef
        where A: BaseActor + 'static
    {
        self.cell.spawn(id, actor)
    }

    pub fn ask_any<T>(&mut self, message: Box<Any + Send>) -> Box<Future<Item = T, Error = ()>>
        where T: Send + 'static
    {
        let (promise, f) = oneshot::channel();
        let ask_ref = self.spawn(Uuid::new_v4(), AskActor::new(promise));
        ask_ref.send_any(message, &ask_ref);
        Box::new(f.map_err(|_| ()))
    }

    pub fn ask<T, R>(&mut self, message: R) -> Box<Future<Item = T, Error = ()>>
        where T: Send + 'static,
              R: Any + Send
    {
        self.ask_any(Box::new(message))
    }
}
