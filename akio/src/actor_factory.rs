use super::{ActorCell, ActorCellHandle, ActorRef, ActorSystem, BaseActor};
use std::collections::HashMap;
use std::collections::hash_map;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorChildren {
    actor_refs: HashMap<Uuid, ActorRef>,
}

impl ActorChildren {
    pub fn new() -> Self {
        Self {
            actor_refs: HashMap::new(),
        }
    }

    pub fn iter(&self) -> hash_map::Values<Uuid, ActorRef> {
        self.actor_refs.values()
    }

    pub(super) fn insert(&mut self, id: Uuid, actor_ref: &ActorRef) {
        match self.actor_refs.insert(id, actor_ref.clone()) {
            Some(_) => panic!("Invalid actor children insert"),
            None => (),
        }
    }
}

pub fn create_actor<A>(system: &ActorSystem, id: Uuid, actor: A) -> ActorRef
where
    A: BaseActor + 'static,
{
    let actor_cell_p = ActorCell::new(system.clone(), id, actor);
    let handle = ActorCellHandle::new(Arc::downgrade(&actor_cell_p));
    handle.on_start();
    system.register_actor(id, actor_cell_p);
    ActorRef::new(handle)
}
