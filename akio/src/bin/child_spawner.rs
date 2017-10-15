#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
extern crate akio;

use akio::prelude::*;

struct TelephoneActor {}

#[actor_impl]
impl TelephoneActor {
    #[actor_api]
    pub fn spawn_next(&mut self, n: u64) {
        if n % 10000 == 0 {
            println!("Spawning {}", n);
        }
        let next_ref = spawn(Uuid::new_v4(), TelephoneActor {});
        next_ref.spawn_next(n + 1);
    }

    #[actor_api]
    pub fn message(&mut self, msg: String) {
        //self.with_children(|children| {
            //children.iter().for_each(|target| {
                //TelephoneActor::from_ref(target.clone()).message(msg.clone())
            //})
        //});
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        let actor_ref = spawn(Uuid::new_v4(), TelephoneActor {});
        actor_ref.spawn_next_with_sender(0, &actor_ref);
        actor_ref.message_with_sender("Yo".to_string(), &actor_ref);
    });
    system.start();
}
