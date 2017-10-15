use typemap::{Key, TypeMap};
use std::marker::PhantomData;

struct ActorKey<T> {
    _actor: PhantomData<T>,
}

impl<T> Key for ActorKey<T>
    where T: Actor
{
    type Value = HashMap<Uuid, Arc<ActorCell>>;
}

struct ActorContainer {
    actors: TypeMap,
}

impl ActorContainer {
    pub fn new() -> Self {
        Self {
            actors: TypeMap::new(),
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

    pub fn get(&self, id: &Uuid)
        -> Option<&Arc<ActorCell<T>>>
        where T: Actor
    {
        self.actors.get::<ActorKey<T>>()
            .and_then(|hash| hash.get(id))
    }
}
