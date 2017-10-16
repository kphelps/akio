use super::{
    Actor,
    ActorCell,
    ActorCellHandle,
    ActorContainer,
    ActorRef,
    ActorResponse,
    Dispatcher,
    MessageHandler,
};
use super::actor_factory::create_actor;
use super::errors::*;
use futures::Future;
use futures::sync::oneshot;
use parking_lot::RwLock;
use std::boxed::FnBox;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorSystem {
    inner: Arc<RwLock<ActorSystemInner>>,
}

struct ActorSystemInner {
    dispatcher: Dispatcher,
    root_actor: Option<ActorRef<GuardianActor>>,
    actors: ActorContainer,
    done_signal: Option<oneshot::Sender<()>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let dispatcher = Dispatcher::new();
        let inner = ActorSystemInner {
            dispatcher: dispatcher,
            root_actor: None,
            actors: ActorContainer::new(),
            done_signal: None,
        };
        let system = Self {
            inner: Arc::new(RwLock::new(inner)),
        };
        system.inner.write().dispatcher.start(system.clone());
        system.inner.write().root_actor = Some(create_actor(
            system.clone(),
            Uuid::new_v4(),
            GuardianActor {},
        ));
        system
    }

    pub fn on_startup<F>(&mut self, f: F)
    where
        F: FnBox() + 'static + Send,
    {
        let root_ref = self.root_actor();
        root_ref.request(GuardianMessage::Execute(Box::new(f)));
    }

    fn root_actor(&self) -> ActorRef<GuardianActor> {
        self.inner.read().root_actor.as_ref().unwrap().clone()
    }

    pub fn start(&self) {
        let (promise, future) = oneshot::channel();
        self.inner.write().done_signal = Some(promise);
        future.wait().expect("Shutdown failed");
    }

    pub fn stop(&self) {
        let system = self.clone();
        if let Some(promise) = self.inner.write().done_signal.take() {
            ::std::thread::spawn(move || {
                let mut locked = system.inner.write();
                locked.dispatcher.join();
                promise.send(()).expect("shutdown failed");
            });
        }
    }

    pub(crate) fn dispatch<A>(&self, actor: ActorCellHandle<A>)
    where
        A: Actor,
    {
        self.inner.read().dispatch(actor);
    }

    pub(crate) fn register_actor<A>(&self, id: Uuid, actor: Arc<ActorCell<A>>)
    where
        A: Actor,
    {
        if let Some(_) = self.inner.write().actors.insert(id.clone(), actor) {
            println!("Replacing existing actor?")
        }
    }

    pub fn deregister_actor<A>(&self, id: &Uuid) -> Result<()>
    where
        A: Actor,
    {
        self.inner
            .write()
            .actors
            .remove(id)
            .ok_or(ErrorKind::InvalidActor(id.clone()).into())
            .map(|_: Arc<ActorCell<A>>| ())
    }

    pub fn get_actor<T>(&self, id: &Uuid) -> Option<ActorRef<T>>
    where
        T: Actor,
    {
        self.inner
            .read()
            .actors
            .get(id)
            .map(|rc| ActorRef::new(ActorCellHandle::new(Arc::downgrade(rc))))
    }
}

impl ActorSystemInner {
    fn dispatch<T>(&self, actor: ActorCellHandle<T>)
    where
        T: Actor,
    {
        self.dispatcher.dispatch(actor)
    }
}

struct GuardianActor {}

enum GuardianMessage {
    Execute(Box<FnBox() + 'static + Send>),
}

impl Actor for GuardianActor {}

impl MessageHandler<GuardianMessage> for GuardianActor {
    type Response = ();

    fn handle(&mut self, message: GuardianMessage) -> ActorResponse<()> {
        match message {
            GuardianMessage::Execute(f) => f(),
        };
        self.done()
    }
}
