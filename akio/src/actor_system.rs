use super::{Actor, ActorCell, ActorCellHandle, ActorRef, Dispatcher};
use super::actor_factory::create_actor;
use super::errors::*;
use parking_lot::RwLock;
use std::boxed::FnBox;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorSystem {
    inner: Arc<RwLock<ActorSystemInner>>,
}

struct ActorSystemInner {
    dispatcher: Dispatcher,
    root_actor: Option<ActorRef>,
    actors: HashMap<Uuid, Arc<ActorCell>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.start();
        let inner = ActorSystemInner {
            dispatcher: dispatcher,
            root_actor: None,
            actors: HashMap::new(),
        };
        let system = Self {
            inner: Arc::new(RwLock::new(inner)),
        };
        system.inner.write().root_actor =
            Some(create_actor(&system, Uuid::new_v4(), GuardianActor {}));
        system
    }

    pub fn on_startup<F>(&mut self, f: F)
    where
        F: FnBox() + 'static + Send,
    {
        let root_ref = self.root_actor();
        root_ref.send_from(GuardianMessage::Execute(Box::new(f)), &root_ref)
    }

    fn root_actor(&self) -> ActorRef {
        self.inner.read().root_actor.as_ref().unwrap().clone()
    }

    pub fn start(&self) {
        ::std::thread::sleep(Duration::from_secs(1000000));
    }

    pub fn stop(self) {
        self.inner.write().dispatcher.join()
    }

    pub fn dispatch(&mut self, actor: ActorCellHandle) {
        self.inner.read().dispatch(actor);
    }

    pub fn register_actor(&self, id: Uuid, actor: Arc<ActorCell>) {
        if let Some(_) = self.inner.write().actors.insert(id.clone(), actor) {
            println!("Replacing existing actor?")
        }
    }

    pub fn deregister_actor(&self, id: &Uuid) -> Result<()> {
        self.inner
            .write()
            .actors
            .remove(id)
            .ok_or(ErrorKind::InvalidActor(id.clone()).into())
            .map(|_| ())
    }

    pub fn get_actor(&self, id: &Uuid) -> Option<ActorRef> {
        self.inner
            .read()
            .actors
            .get(id)
            .map(|rc| ActorRef::new(ActorCellHandle::new(Arc::downgrade(rc))))
    }
}

impl ActorSystemInner {
    fn dispatch(&self, actor: ActorCellHandle) {
        self.dispatcher.dispatch(actor)
    }
}

struct GuardianActor {}

enum GuardianMessage {
    Execute(Box<FnBox() + 'static + Send>),
}

impl Actor for GuardianActor {
    type Message = GuardianMessage;

    fn handle_message(&mut self, message: Self::Message) {
        match message {
            GuardianMessage::Execute(f) => f(),
        }
    }
}
