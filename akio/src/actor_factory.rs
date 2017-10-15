use super::{Actor, ActorCell, ActorCellHandle, ActorRef, ActorSystem};
use std::sync::Arc;
use uuid::Uuid;

pub fn create_actor<A>(system: ActorSystem, id: Uuid, actor: A) -> ActorRef<A>
where
    A: Actor + 'static,
{
    let actor_cell_p = ActorCell::new(system.clone(), id, actor);
    let handle = ActorCellHandle::new(Arc::downgrade(&actor_cell_p));
    handle.on_start();
    system.register_actor(id, actor_cell_p);
    ActorRef::new(handle)
}
