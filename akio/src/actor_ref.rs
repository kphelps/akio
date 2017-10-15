use super::{context, Actor, ActorCellHandle, AskActor, SystemMessage};
use futures::prelude::*;
use futures::sync::oneshot;
use std::any::Any;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorRef<A> {
    cell: ActorCellHandle<A>,
}

impl<A> ActorRef<A>
    where A: Actor
{
    pub fn new(cell: ActorCellHandle<A>) -> Self {
        Self {
            cell: cell,
        }
    }

    pub fn exists(&self) -> bool {
        self.cell.exists()
    }

    pub fn id(&self) -> Uuid {
        self.cell.id()
    }

    pub fn send<T>(&self, message: T)
        where A: MessageHandler<T>
    {
        context::with(|ctx| self.send_from(message, &ctx.sender))
    }

    pub fn send_from<B, T>(&self, message: T, sender: &ActorRef<B>)
        where B: Actor,
              A: MessageHandler<T>
    {
        self.cell.enqueue_message(message, sender.clone())
    }

    fn system_send(&self, message: SystemMessage) {
        self.cell.enqueue_system_message(message)
    }

    pub fn spawn<B>(&self, id: Uuid, actor: B) -> ActorRef<B>
    where
        B: Actor
    {
        self.cell.spawn(id, actor)
    }

    pub fn ask<R>(&self, message: R) -> impl Future<Item = A::Response, Error = ()>
    where
        A: MessageHandler<R>
    {
        let (promise, f) = oneshot::channel();
        let id = Uuid::new_v4();
        let ask_ref = self.spawn(id, AskActor::new(promise));
        self.send_from(message, &ask_ref);
        f.map_err(|_| ())
    }

    pub fn stop(&self) -> impl Future<Item = (), Error = ()> {
        let (promise, future) = oneshot::channel();
        self.system_send(SystemMessage::Stop(promise));
        future.map_err(|_| ())
    }
}
