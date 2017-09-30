use tokio_core::reactor::Remote;
use futures::prelude::*;
use futures::sync::mpsc;
use super::{ActorCell, ActorEvent, ActorRef, BaseActor};
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

    fn remote(&self) -> Remote;

    fn enqueuer(&self) -> mpsc::Sender<ActorEvent>;

    fn spawn<A>(&mut self, id: Uuid, actor: A) -> Box<Future<Item = ActorRef, Error = ()> + Send>
        where A: BaseActor + 'static
    {
        let (actor_ref, f) = create_actor(id, actor, self.enqueuer(), self.remote());
        self.children().insert(id, &actor_ref);
        f
    }
}

pub fn create_actor<A>(id: Uuid,
                       actor: A,
                       enqueuer: mpsc::Sender<ActorEvent>,
                       remote: Remote)
                       -> (ActorRef, Box<Future<Item = ActorRef, Error = ()> + Send>)
    where A: BaseActor + 'static
{
    let actor_cell = ActorCell::new(id, actor, enqueuer.clone(), remote);
    let actor_ref = actor_cell.actor_ref();
    let inner_actor_ref = actor_cell.actor_ref();
    let f = Box::new(enqueuer
                         .send(ActorEvent::ActorIdle(actor_cell))
                         .map(|_| inner_actor_ref)
                         .map_err(|_| ()));
    (actor_ref, f)
}
