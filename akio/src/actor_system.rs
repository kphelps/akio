use super::{Actor, ActorCell, ActorChildren, ActorFactory, ActorRef, Dispatcher};
use super::actor_factory::create_actor;
use futures::future::{Executor, ExecuteError};
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::boxed::FnBox;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tokio_core::reactor::{Core, Handle, Remote};
use uuid::Uuid;

pub enum ActorEvent {
    MailboxReady(Uuid),
    ActorIdle(ActorCell),
}

enum ActorStatus {
    Idle(ActorCell),
    Scheduled(),
}

pub struct ActorSystem {
    core: Core,
    inner: Rc<RefCell<ActorSystemInner>>,
    enqueuer: mpsc::Sender<ActorEvent>,
    event_queue: mpsc::Receiver<ActorEvent>,
    root_actor: ActorRef,
}

struct ActorSystemInner {
    start: Instant,
    counter: u64,
    actors: HashMap<Uuid, ActorStatus>,
    dispatcher: Dispatcher,
    handle: Handle,
}

impl ActorSystem {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let core = Core::new().expect("Failed to create event loop");
        let inner = ActorSystemInner {
            start: Instant::now(),
            counter: 0,
            actors: HashMap::new(),
            dispatcher: Dispatcher::new(sender.clone()),
            handle: core.handle(),
        };
        let (root_actor, root_actor_f) = create_actor(Uuid::new_v4(),
                                                      GuardianActor {},
                                                      sender.clone(),
                                                      core.remote());
        core.handle().spawn(root_actor_f.map(|_| ()));
        Self {
            core: core,
            inner: Rc::new(RefCell::new(inner)),
            enqueuer: sender,
            event_queue: receiver,
            root_actor: root_actor,
        }
    }

    pub fn on_startup<F>(&mut self, f: F)
        where F: FnBox() + Send + 'static
    {
        self.root_actor
            .send(GuardianMessage::Execute(Box::new(f)), &self.root_actor)
    }

    pub fn start(mut self) {
        let inner = self.inner.clone();
        let stream = self.event_queue
            .map(|event| inner.borrow_mut().handle_event(event))
            .map_err(|_| println!("Err"))
            .for_each(|_| Ok(()));
        self.inner.borrow_mut().start_dispatcher();
        self.core.run(stream).expect("Failure");
        Rc::try_unwrap(self.inner)
            .ok()
            .expect("Failed shutting down system")
            .into_inner()
            .join();
    }
}

impl ActorSystemInner {
    fn handle_event(&mut self, event: ActorEvent) {
        match event {
            ActorEvent::MailboxReady(id) => self.dispatch(&id),
            ActorEvent::ActorIdle(actor_cell) => self.undispatch(actor_cell),
        };
    }

    fn undispatch(&mut self, actor_cell: ActorCell) {
        match self.actors
                  .insert(actor_cell.id(), ActorStatus::Idle(actor_cell)) {
            Some(ActorStatus::Idle(_)) => panic!("Idle actor replacing actor??"),
            Some(ActorStatus::Scheduled()) => (),
            None => (),
        }
    }

    fn dispatch(&mut self, id: &Uuid) {
        self.counter += 1;
        if self.counter % 1000 == 0 {
            let dt = (Instant::now() - self.start).as_secs();
            if dt > 0 {
                let rate = self.counter / dt;
                println!("Dispatch {} ({}/s)", self.counter, rate);
            }
        }
        match self.actors.insert(*id, ActorStatus::Scheduled()) {
            Some(ActorStatus::Idle(actor)) => {
                let f = self.dispatcher.dispatch(actor);
                self.handle.execute(f).expect("Failed to dispatch");
            }
            None => panic!("Attempted to schedule a non existant actor"),
            _ => (),
        }
    }

    fn start_dispatcher(&mut self) {
        self.dispatcher.start();
    }

    fn join(self) {
        self.dispatcher.join();
    }
}

impl<F> Executor<F> for ActorSystem
    where F: Future<Item = (), Error = ()> + 'static
{
    fn execute(&self, future: F) -> Result<(), ExecuteError<F>> {
        self.core.execute(future)
    }
}

struct GuardianActor {}

enum GuardianMessage {
    Execute(Box<FnBox() + Send>),
}

impl Actor for GuardianActor {
    type Message = GuardianMessage;

    fn handle_message(&mut self, message: Self::Message) {
        match message {
            GuardianMessage::Execute(f) => f(),
        }
    }
}
