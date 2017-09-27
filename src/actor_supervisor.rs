use super::{BaseActor, ActorRef};
use uuid::Uuid;

pub trait ActorSupervisor {
    fn spawn<A>(&mut self, id: Uuid, actor: A) -> Option<ActorRef> where A: BaseActor + 'static;
}
