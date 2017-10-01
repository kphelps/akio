use super::{Actor, ActorCell, ActorRef, Dispatcher};
use super::actor_factory::create_actor;
use futures::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::boxed::FnBox;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActorSystem {
    inner: Arc<Mutex<ActorSystemInner>>,
}

struct ActorSystemInner {
    start: Instant,
    counter: u64,
    dispatcher: Dispatcher,
    root_actor: Option<ActorRef>,
}

impl ActorSystem {
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher::new();
        dispatcher.start();
        let inner = ActorSystemInner {
            start: Instant::now(),
            counter: 0,
            dispatcher: dispatcher,
            root_actor: None,
        };
        let system = Self { inner: Arc::new(Mutex::new(inner)) };
        system.inner.lock().root_actor =
            Some(create_actor(&system, Uuid::new_v4(), GuardianActor {}));
        system
    }

    pub fn on_startup<F>(&mut self, f: F)
        where F: FnBox() -> Box<Future<Item = (), Error = ()>> + Send + 'static
    {
        println!("Send");
        let root_ref = self.root_actor();
        root_ref.send(GuardianMessage::Execute(Box::new(f)), &root_ref)
    }

    fn root_actor(&self) -> ActorRef {
        self.inner.lock().root_actor.as_ref().unwrap().clone()
    }

    pub fn start(&self) {
        start_deadlock_detector();
        ::std::thread::sleep_ms(1000000);
    }

    pub fn dispatch(&mut self, actor: ActorCell) {
        self.inner.lock().dispatch(actor);
    }
}

impl ActorSystemInner {
    fn dispatch(&mut self, actor: ActorCell) {
        self.counter += 1;
        if self.counter % 10000 == 0 {
            println!("Counter: {}", self.counter);
        }
        if self.counter > 1000000 {
            let dt = (Instant::now() - self.start).as_secs();
            if dt > 0 {
                let rate = self.counter / dt;
                println!("Dispatch {} ({}/s)", self.counter, rate);
                ::std::process::exit(0);
            }
        }
        self.dispatcher.dispatch(actor)
    }
}

struct GuardianActor {}

enum GuardianMessage {
    Execute(Box<FnBox() -> Box<Future<Item = (), Error = ()>> + Send>),
}

impl Actor for GuardianActor {
    type Message = GuardianMessage;

    fn handle_message(&mut self, message: Self::Message) -> Box<Future<Item = (), Error = ()>> {
        match message {
            GuardianMessage::Execute(f) => f(),
        }
    }
}


fn start_deadlock_detector() {
    {
        // only for #[cfg]
        use std::thread;
        use std::time::Duration;
        use parking_lot::deadlock;

        // Create a background thread which checks for deadlocks every 10s
        thread::spawn(move || loop {
                          println!("Checking");
                          thread::sleep(Duration::from_secs(10));
                          let deadlocks = deadlock::check_deadlock();
                          if deadlocks.is_empty() {
                              continue;
                          }

                          println!("{} deadlocks detected", deadlocks.len());
                          for (i, threads) in deadlocks.iter().enumerate() {
                              println!("Deadlock #{}", i);
                              for t in threads {
                                  println!("Thread Id {:#?}", t.thread_id());
                                  println!("{:#?}", t.backtrace());
                              }
                          }
                      });
    }
}
