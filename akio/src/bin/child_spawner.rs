extern crate akio;
extern crate akio_syntax;
extern crate futures;
extern crate uuid;

use akio::*;
use akio_syntax::*;
use futures::prelude::*;
use futures::future::Executor;
use uuid::Uuid;

actor! {
    TelephoneActor,

    message Spawn(val: u64) {
        match val {
            0 => (),
            n => {
                println!("Spawning {}", n);
                let id = Uuid::new_v4();
                let f = TelephoneActor::spawn(id.clone())
                    .map(move |target_ref| { target_ref.spawn(n - 1); });
                context::execute(f).unwrap();
            }
        }
    }

    message Message(msg: String) {
        println!("{}", msg);
        self.with_children(|children| {
            children
                .iter()
                .for_each(|target| {
                    TelephoneActor::from_ref(&target).message(msg.clone())
                })
        })
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
                          let f = TelephoneActor::spawn(Uuid::new_v4()).map(|actor_ref| {
            actor_ref.spawn_with_sender(10, &actor_ref);
            actor_ref.message_with_sender("Yo".to_string(), &actor_ref);
        });
                          context::execute(f);
                      });
    system.start();
}
