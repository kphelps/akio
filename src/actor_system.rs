use super::{Actor, ActorCell, ActorRef, ActorSupervisor, BaseActorCell};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use tokio_core::reactor::{Core, Remote};
use uuid::Uuid;

enum ActorEvent {
    MailboxReady(Uuid),
}

pub struct ActorSystem {
    core: Core,
    inner: Rc<RefCell<ActorSystemInner>>,
    _enqueuer: mpsc::Sender<ActorEvent>,
    event_queue: mpsc::Receiver<ActorEvent>,
}

#[derive(Clone)]
pub struct ActorSystemHandle {
    remote_handle: Remote,
    inner: Weak<RefCell<ActorSystemInner>>,
}

struct ActorSystemInner {
    enqueuer: mpsc::Sender<ActorEvent>,
    actors: HashMap<Uuid, Box<RefCell<BaseActorCell>>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let core = Core::new().expect("Failed to create event loop");
        let inner = ActorSystemInner {
            enqueuer: sender.clone(),
            actors: HashMap::new(),
        };
        Self {
            core: core,
            inner: Rc::new(RefCell::new(inner)),
            _enqueuer: sender,
            event_queue: receiver,
        }
    }

    pub fn start(mut self) {
        let inner = self.inner;
        let stream = self.event_queue
            .map(|event| inner.borrow_mut().handle_event(event))
            .map_err(|_| println!("Err"))
            .for_each(|_| Ok(()));
        self.core.run(stream).expect("Failure");
    }

    fn handle(&self) -> ActorSystemHandle {
        ActorSystemHandle {
            remote_handle: self.core.remote(),
            inner: Rc::downgrade(&self.inner),
        }
    }
}

impl ActorSystemInner {
    fn handle_event(&mut self, event: ActorEvent) {
        match event {
            ActorEvent::MailboxReady(id) => {
                self.actors
                    .get(&id)
                    .expect("Actor mailbox does not exist")
                    .borrow_mut()
                    .process_message()
            }
        };
    }
}

impl ActorSupervisor for ActorSystem {
    fn spawn<A, T>(&mut self, id: Uuid, actor: A) -> Option<ActorRef<T>>
        where A: Actor<T> + 'static,
              T: 'static
    {
        let actor_cell = ActorCell::new(id, actor, self.handle());
        let handle = actor_cell.handle();
        if self.inner
               .borrow_mut()
               .actors
               .insert(id, Box::new(RefCell::new(actor_cell)))
               .is_some() {
            None
        } else {
            Some(ActorRef::new(handle))
        }
    }
}

impl ActorSystemHandle {
    pub fn mailbox_ready(&self, id: Uuid) {
        let f = self.enqueuer()
            .send(ActorEvent::MailboxReady(id))
            .map(|_| ())
            .map_err(|_| ());
        self.remote_handle.execute(f).expect("readying mailbox")
    }

    fn enqueuer(&self) -> mpsc::Sender<ActorEvent> {
        let inner = self.inner.upgrade().expect("Failed to get system");
        let sender = inner.borrow_mut().enqueuer.clone();
        sender
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
        fn handle_message(&self, message: ExampleMessage) {
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
