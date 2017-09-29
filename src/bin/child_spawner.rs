extern crate akio;
extern crate futures;
extern crate uuid;

use akio::*;
use futures::prelude::*;
use futures::future;
use futures::future::Executor;
use std::iter;
use uuid::Uuid;

enum Message {
    Spawn(u64),
    Message(String),
}

struct TelephoneActor {}

impl Actor for TelephoneActor {
    type Message = Message;

    fn handle_message(&mut self, context: &mut ActorContext, message: Message) {
        match message {
            Message::Spawn(0) => (),
            Message::Spawn(n) => {
                println!("Spawning {}", n);
                let id = Uuid::new_v4();
                let f = context
                    .spawn(id.clone(), TelephoneActor {})
                    .map(move |target_ref| {
                             target_ref.send(Message::Spawn(n - 1), &target_ref);
                         })
                    .map(|_| ());
                context.execute(f).unwrap();
            }
            Message::Message(msg) => {
                println!("{}", msg);
                let me = context.self_ref.clone();
                context
                    .children
                    .iter()
                    .for_each(|target| target.send(Message::Message(msg.clone()), &me))
            }
        }
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let f = system
        .spawn(Uuid::new_v4(), TelephoneActor {})
        .map(|actor_ref| {
                 actor_ref.send(Message::Spawn(10), &actor_ref);
                 actor_ref.send(Message::Message("Yo Dawg".to_string()), &actor_ref);
             });
    system.execute(f).unwrap();
    system.start();
}
