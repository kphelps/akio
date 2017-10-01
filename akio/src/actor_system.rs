use super::{Actor, ActorCell, ActorRef, Dispatcher};
use super::actor_factory::create_actor;
use parking_lot::RwLock;
use std::boxed::FnBox;
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
}

impl ActorSystem {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.start();
        let inner = ActorSystemInner {
            dispatcher: dispatcher,
            root_actor: None,
        };
        let system = Self { inner: Arc::new(RwLock::new(inner)) };
        system.inner.write().root_actor =
            Some(create_actor(&system, Uuid::new_v4(), GuardianActor {}));
        system
    }

    pub fn on_startup<F>(&mut self, f: F)
        where F: FnBox() + 'static + Send
    {
        let root_ref = self.root_actor();
        root_ref.send(GuardianMessage::Execute(Box::new(f)), &root_ref)
    }

    fn root_actor(&self) -> ActorRef {
        self.inner.read().root_actor.as_ref().unwrap().clone()
    }

    pub fn start(&self) {
        ::std::thread::sleep(Duration::from_secs(1000000));
    }

    pub fn stop(self) {
        Arc::try_unwrap(self.inner)
            .ok()
            .unwrap()
            .into_inner()
            .dispatcher
            .join()
    }

    pub fn dispatch(&mut self, actor: ActorCell) {
        self.inner.read().dispatch(actor);
    }
}

impl ActorSystemInner {
    fn dispatch(&self, actor: ActorCell) {
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
        println!("Process?");
        match message {
            GuardianMessage::Execute(f) => f(),
        }
    }
}
