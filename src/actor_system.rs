use super::{Actor, ActorCell, ActorRef, ActorSupervisor, BaseActorCell};
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use tokio_core::reactor::Core;
use uuid::Uuid;

pub enum ActorEvent {
    MailboxReady(Uuid),
}

pub struct ActorSystem {
    core: Core,
    inner: Rc<RefCell<ActorSystemInner>>,
    enqueuer: mpsc::Sender<ActorEvent>,
    event_queue: mpsc::Receiver<ActorEvent>,
}

struct ActorSystemInner {
    actors: HashMap<Uuid, Box<RefCell<BaseActorCell>>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let core = Core::new().expect("Failed to create event loop");
        let inner = ActorSystemInner { actors: HashMap::new() };
        Self {
            core: core,
            inner: Rc::new(RefCell::new(inner)),
            enqueuer: sender,
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
              T: Clone + 'static
    {
        let actor_cell = ActorCell::new(id, actor, self.enqueuer.clone(), self.core.remote());
        let actor_ref = actor_cell.actor_ref();
        if self.inner
               .borrow_mut()
               .actors
               .insert(id, Box::new(RefCell::new(actor_cell)))
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
