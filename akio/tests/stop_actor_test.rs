#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
extern crate akio;

mod common;

use akio::prelude::*;
use common::*;

#[test]
fn test_create_get_actor() {
    with_actor_system(|system| {
        let actor_ref = TestActor::new().start();
        let system_actor_ref = system.get_actor::<TestActor>(&actor_ref.id());
        assert!(system_actor_ref.is_some());
        assert_eq!(system_actor_ref.unwrap().id(), actor_ref.id());
    })
}

#[test]
fn test_create_stop_get_actor() {
    with_actor_system_async(|system| {
        let actor_ref = TestActor::new().start();
        let id = actor_ref.id();
        actor_ref
            .clone()
            .stop()
            .map_err(|_| println!("Failed to stop"))
            .map(move |_| {
                assert!(system.get_actor::<TestActor>(&id).is_none());
                assert!(!actor_ref.exists());
            })
    })
}
