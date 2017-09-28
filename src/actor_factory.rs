use tokio_core::reactor::Remote;
use futures::prelude::*;
use futures::sync::mpsc;
use super::{ActorCell, ActorEvent, ActorRef, BaseActor};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ActorChildren {
    actor_refs: HashMap<Uuid, ActorRef>,
}

impl ActorChildren {
    pub fn new() -> Self {
        Self { actor_refs: HashMap::new() }
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

    fn spawn<A>(&mut self, id: Uuid, actor: A) -> Box<Future<Item = ActorRef, Error = ()>>
        where A: BaseActor + 'static
    {
        let actor_cell = ActorCell::new(id, actor, self.enqueuer(), self.remote());
        let actor_ref = actor_cell.actor_ref();
        self.children().insert(id, &actor_ref);
        Box::new(self.enqueuer()
                     .send(ActorEvent::ActorIdle(actor_cell))
                     .map(|_| actor_ref)
                     .map_err(|_| ()))
    }
}
