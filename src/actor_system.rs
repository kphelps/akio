use super::{BaseActor, ActorCell, ActorRef, ActorSupervisor, Dispatcher};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tokio_core::reactor::{Core, Handle};
use uuid::Uuid;

pub enum ActorEvent {
    MailboxReady(Uuid),
    ActorIdle(ActorCell),
}

pub struct ActorSystem {
    core: Core,
    inner: Rc<RefCell<ActorSystemInner>>,
    enqueuer: mpsc::Sender<ActorEvent>,
    event_queue: mpsc::Receiver<ActorEvent>,
}

struct ActorSystemInner {
    start: Instant,
    counter: u64,
    actors: HashMap<Uuid, ActorCell>,
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
        Self {
            core: core,
            inner: Rc::new(RefCell::new(inner)),
            enqueuer: sender,
            event_queue: receiver,
        }
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
            ActorEvent::ActorIdle(actor_cell) => {
                match self.actors.insert(actor_cell.id(), actor_cell) {
                    Some(_) => println!("Idle actor replacing actor??"),
                    None => (),
                }
            }
        };
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
        match self.actors.remove(&id) {
            Some(actor) => {
                let f = self.dispatcher.dispatch(actor);
                self.handle.execute(f).expect("Failed to dispatch");
            }
            None => (),
        }
    }

    fn start_dispatcher(&mut self) {
        self.dispatcher.start();
    }

    fn join(self) {
        self.dispatcher.join();
    }
}

impl ActorSupervisor for ActorSystem {
    fn spawn<A>(&mut self, id: Uuid, actor: A) -> Option<ActorRef>
        where A: BaseActor + 'static
    {
        let actor_cell = ActorCell::new(id, actor, self.enqueuer.clone(), self.core.remote());
        let actor_ref = actor_cell.actor_ref();
        if self.inner
               .borrow_mut()
               .actors
               .insert(id, actor_cell)
               .is_some() {
            None
        } else {
            Some(actor_ref)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum ExampleMessage {
        Test(),
    }

    struct ExampleActor {}

    impl Actor<ExampleMessage> for ExampleActor {
        fn handle_message(&mut self, message: ExampleMessage) {
            match message {
                ExampleMessage::Test() => (),
            }
        }
    }

    #[test]
    fn test_actor_system() {
        let mut system = ActorSystem::new();
        let maybe_ref = system.spawn(Uuid::new_v4(), ExampleActor {});
        assert!(maybe_ref.is_some());
    }

    #[test]
    fn test_actor_system_duplicate_actor_id() {
        let id = Uuid::new_v4();
        let mut system = ActorSystem::new();
        system.spawn(id, ExampleActor {});
        let maybe_ref = system.spawn(id, ExampleActor {});
        assert!(maybe_ref.is_none());
    }
}
