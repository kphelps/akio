use super::{ActorEvent, ActorRef};
use futures::sync::mpsc;
use tokio_core::reactor::Remote;

pub struct ActorContext<T> {
    pub self_ref: ActorRef<T>,
    pub enqueuer: mpsc::Sender<ActorEvent>,
    // TODO: This should probably be just a Handle if actors are only assigned
    // to a signle thread.
    pub remote_handle: Remote,
}

impl<T> ActorContext<T> {
    pub fn new(self_ref: ActorRef<T>,
               enqueuer: mpsc::Sender<ActorEvent>,
               remote_handle: Remote)
               -> Self {
        Self {
            self_ref: self_ref,
            enqueuer: enqueuer,
            remote_handle: remote_handle,
        }
    }
}
