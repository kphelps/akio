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

    message Spawn(n: u64) {
        if n % 10000 == 0 {
            println!("Spawning {}", n);
        }
        let next_ref = TelephoneActor::spawn(Uuid::new_v4());
        next_ref.spawn(n + 1);
    }

    message Message(msg: String) {
        self.with_children(|children| {
            children
                .iter()
                .for_each(|target| {
                    TelephoneActor::from_ref(&target).message(msg.clone())
                })
        });
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
                          let actor_ref = TelephoneActor::spawn(Uuid::new_v4());
                          actor_ref.spawn_with_sender(0, &actor_ref);
                          actor_ref.message_with_sender("Yo".to_string(), &actor_ref);
                      });
    system.start();
}
