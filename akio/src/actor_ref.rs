use super::{Actor, ActorCellHandle, ActorResponse, MessageHandler, SystemMessage};
use futures::prelude::*;
use futures::sync::oneshot;
use std::clone::Clone;
use uuid::Uuid;

pub struct ActorRef<A> {
    cell: ActorCellHandle<A>,
}

impl<A> Clone for ActorRef<A>
    where A: Actor
{
    fn clone(&self) -> Self {
        Self::new(self.cell.clone())
    }
}

impl<A> ActorRef<A>
    where A: Actor
{
    pub(crate) fn new(cell: ActorCellHandle<A>) -> Self {
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

    pub fn request<T>(&self, message: T) ->
        impl Future<
            Item = ActorResponse<A::Response>,
            Error = ()
        >
        where A: MessageHandler<T>,
              T: Send + 'static
    {
        let (promise, future) = oneshot::channel();
        self.cell.enqueue_message(message, Some(promise));
        future.map_err(|_| ())
    }

    pub fn send<T>(&self, message: T)
        where A: MessageHandler<T>,
              T: Send + 'static
    {
        self.cell.enqueue_message(message, None);
    }

    fn system_send(&self, message: SystemMessage) {
        self.cell.enqueue_system_message(message)
    }

    pub fn stop(&self) -> impl Future<Item = (), Error = ()> {
        let (promise, future) = oneshot::channel();
        self.system_send(SystemMessage::Stop(promise));
        future.map_err(|_| ())
    }
}
