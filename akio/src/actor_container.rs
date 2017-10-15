use super::{Actor, ActorCell};
use typemap::{Key, ShareMap};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use uuid::Uuid;

struct ActorKey<T> {
    _actor: PhantomData<T>,
}

impl<T> Key for ActorKey<T>
    where T: Actor
{
    type Value = HashMap<Uuid, Arc<ActorCell<T>>>;
}

pub(crate) struct ActorContainer {
    actors: ShareMap,
}

impl ActorContainer {
    pub fn new() -> Self {
        Self {
            actors: ShareMap::custom(),
        }
    }

    pub fn insert<T>(&mut self, id: Uuid, actor: Arc<ActorCell<T>>)
        -> Option<Arc<ActorCell<T>>>
        where T: Actor
    {
        self.actors.entry::<ActorKey<T>>()
            .or_insert_with(HashMap::new)
            .insert(id, actor)
    }

    pub fn remove<T>(&mut self, id: &Uuid)
        -> Option<Arc<ActorCell<T>>>
        where T: Actor
    {
        self.actors.get_mut::<ActorKey<T>>()
            .and_then(|hash| hash.remove(id))
    }

    pub fn get<T>(&self, id: &Uuid)
        -> Option<&Arc<ActorCell<T>>>
        where T: Actor
    {
        self.actors.get::<ActorKey<T>>()
            .and_then(|hash| hash.get(id))
    }
}
