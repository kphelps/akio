use super::{ActorCellHandle, ActorChildren, AskActor, BaseActor, context};
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

    pub fn id(&self) -> &Uuid {
        self.cell.id()
    }

    pub fn send<T: Any + Send>(&self, message: T) {
        context::with(|ctx| self.send_from(message, &ctx.sender))
    }

    pub fn send_from<T: Any + Send>(&self, message: T, sender: &ActorRef) {
        self.send_any_from(Box::new(message), sender)
    }

    pub fn send_any_from(&self, message: Box<Any + Send>, sender: &ActorRef) {
        self.cell.enqueue_message(message, sender.clone())
    }

    pub fn spawn<A>(&self, id: Uuid, actor: A) -> ActorRef
        where A: BaseActor + 'static
    {
        self.cell.spawn(id, actor)
    }

    pub fn with_children<F, R>(&self, f: F) -> R
        where F: FnOnce(&ActorChildren) -> R
    {
        self.cell.with_children(f)
    }

    pub fn ask_any<T>(&self, message: Box<Any + Send>) -> Box<Future<Item = T, Error = ()> + Send>
        where T: Send + 'static
    {
        let (promise, f) = oneshot::channel();
        let id = Uuid::new_v4();
        let ask_ref = self.spawn(id, AskActor::new(promise));
        self.send_any_from(message, &ask_ref);
        Box::new(f.map_err(|_| ()))
    }

    pub fn ask<T, R>(&self, message: R) -> Box<Future<Item = T, Error = ()> + Send>
        where T: Send + 'static,
              R: Any + Send
    {
        self.ask_any(Box::new(message))
    }
}
