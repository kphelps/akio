use super::{Actor, ActorRef};
use uuid::Uuid;

pub trait ActorSupervisor {
    fn spawn<A, T>(&mut self, id: Uuid, actor: A) -> Option<ActorRef<T>>
        where A: Actor<T> + 'static,
              T: Clone + 'static;
}
