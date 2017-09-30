extern crate akio;
extern crate akio_syntax;
extern crate futures;
extern crate uuid;

use akio::*;
use akio_syntax::*;
use futures::prelude::*;
use futures::future;
use futures::future::Executor;
use std::iter;
use uuid::Uuid;

actor! {
    TelephoneActor,

    message Spawn(val: u64) {
        match val {
            0 => (),
            n => {
                println!("Spawning {}", n);
                let id = Uuid::new_v4();
                let f = context
                    .spawn(id.clone(), TelephoneActor {})
                    .map(move |target_ref| {
                             target_ref.send(TelephoneActorMessage::Spawn(n - 1), &target_ref);
                         })
                    .map(|_| ());
                context.execute(f).unwrap();
            }
        }
    }

    message Message(msg: String) {
        println!("{}", msg);
        let me = context.self_ref.clone();
        context
            .children
            .iter()
            .for_each(|target| target.send(TelephoneActorMessage::Message(msg.clone()), &me))
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let f = system
        .spawn(Uuid::new_v4(), TelephoneActor {})
        .map(|actor_ref| {
                 actor_ref.send(TelephoneActorMessage::Spawn(10), &actor_ref);
                 actor_ref.send(TelephoneActorMessage::Message("Yo Dawg".to_string()),
                                &actor_ref);
             });
    system.execute(f).unwrap();
    system.start();
}
