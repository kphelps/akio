use super::{ActorCell, ActorRef, ActorSystem, BaseActor};
use std::collections::HashMap;
use std::collections::hash_map;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorChildren {
    actor_refs: HashMap<Uuid, ActorRef>,
}

impl ActorChildren {
    pub fn new() -> Self {
        Self { actor_refs: HashMap::new() }
    }

    pub fn iter(&self) -> hash_map::Values<Uuid, ActorRef> {
        self.actor_refs.values()
    }

    pub(self) fn insert(&mut self, id: Uuid, actor_ref: &ActorRef) {
        match self.actor_refs.insert(id, actor_ref.clone()) {
            Some(_) => panic!("Invalid actor children insert"),
            None => (),
        }
    }
}

pub trait ActorFactory {
    fn children(&mut self) -> &mut ActorChildren;

    fn spawn<A>(&mut self, system: &ActorSystem, id: Uuid, actor: A) -> ActorRef
        where A: BaseActor + 'static
    {
        let actor_ref = create_actor(system, id, actor);
        self.children().insert(id, &actor_ref);
        actor_ref
    }
}

pub fn create_actor<A>(system: &ActorSystem, id: Uuid, actor: A) -> ActorRef
    where A: BaseActor + 'static
{
    let actor_cell = ActorCell::new(system.clone(), id, actor);
    let actor_ref = ActorRef::new(actor_cell.clone());
    actor_ref
}
