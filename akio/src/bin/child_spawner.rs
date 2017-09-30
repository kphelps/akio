extern crate akio;
extern crate akio_syntax;
extern crate futures;
extern crate uuid;

use akio::*;
use akio_syntax::*;
use futures::prelude::*;
use futures::future;
use uuid::Uuid;

actor! {
    TelephoneActor,

    message Spawn(val: u64) {
        let f: Box<Future<Item = (), Error = ()>> = match val {
            0 => Box::new(future::ok(())),
            n => {
                println!("Spawning {}", n);
                let id = Uuid::new_v4();
                Box::new(TelephoneActor::spawn(id.clone())
                    .map(move |target_ref| { target_ref.spawn(n - 1); }))
            }
        };
        f
    }

    message Message(msg: String) {
        println!("{}", msg);
        self.with_children(|children| {
            children
                .iter()
                .for_each(|target| {
                    TelephoneActor::from_ref(&target).message(msg.clone())
                })
        });
        Box::new(future::ok(()))
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| -> Box<Future<Item = (), Error = ()>> {
                          Box::new(TelephoneActor::spawn(Uuid::new_v4()).map(move |actor_ref| {
            actor_ref.spawn_with_sender(10, &actor_ref);
            actor_ref.message_with_sender("Yo".to_string(), &actor_ref);
        }))
                      });
    system.start();
}
