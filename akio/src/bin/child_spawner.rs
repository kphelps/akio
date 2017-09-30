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
                let f = TelephoneActor::spawn(context, id.clone())
                    .map(move |target_ref| { target_ref.spawn(n - 1); });
                context.execute(f).unwrap();
            }
        }
    }

    message Message(msg: String) {
        println!("{}", msg);
        context
            .children
            .iter()
            .for_each(|target| TelephoneActor::from_ref(&target).message(msg.clone()))
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let f = TelephoneActor::spawn(&mut system, Uuid::new_v4()).map(|actor_ref| {
        actor_ref.spawn(10);
        actor_ref.message_with_sender("Yo".to_string(), &actor_ref);
    });
    system.execute(f).unwrap();
    system.start();
}
