use super::{ActorEvent, ActorRef};
use futures::sync::mpsc;
use futures::future::{Executor, Future, ExecuteError};
use std::any::Any;
use tokio_core::reactor::Remote;

pub struct ActorContext {
    pub self_ref: ActorRef,
    pub enqueuer: mpsc::Sender<ActorEvent>,
    pub remote_handle: Remote,
    pub sender: ActorRef,
}

impl ActorContext {
    pub fn new(self_ref: ActorRef,
               enqueuer: mpsc::Sender<ActorEvent>,
               remote_handle: Remote)
               -> Self {
        Self {
            self_ref: self_ref.clone(),
            enqueuer: enqueuer,
            remote_handle: remote_handle,
            sender: self_ref,
        }
    }

    pub fn reply<T: Any + Send>(&self, message: T) {
        self.sender.send(message, &self.self_ref)
    }
}

impl<F> Executor<F> for ActorContext
    where F: Future<Item = (), Error = ()> + Send + 'static
{
    fn execute(&self, future: F) -> Result<(), ExecuteError<F>> {
        match self.remote_handle.handle() {
            Some(handle) => handle.execute(future),
            None => self.remote_handle.execute(future),
        }
    }
}
