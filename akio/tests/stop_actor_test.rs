#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
extern crate akio;

mod common;

use akio::prelude::*;
use common::*;

#[test]
fn test_create_actor_id() {
    with_actor_system(|_| {
        let id = Uuid::new_v4();
        let actor_ref = spawn(id.clone(), TestActor::new());
        assert_eq!(id, actor_ref.id());
    })
}

#[test]
fn test_create_get_actor() {
    with_actor_system(|system| {
        let id = Uuid::new_v4();
        let actor_ref = spawn(id.clone(), TestActor::new());
        let system_actor_ref = system.get_actor(&id);
        assert!(system_actor_ref.is_some());
        assert_eq!(system_actor_ref.unwrap().id(), actor_ref.id());
    })
}

#[test]
fn test_create_stop_get_actor() {
    with_actor_system_async(|system| {
        let id = Uuid::new_v4();
        let actor_ref = spawn(id.clone(), TestActor::new());
        actor_ref.clone().stop().map_err(|_| ()).map(move |_| {
            assert!(system.get_actor(&id).is_none());
            assert!(!actor_ref.exists());
        })
    })
}

#[test]
fn test_stop_actor_with_children() {
    with_actor_system_async(|system| {
        let parent_id = Uuid::new_v4();
        let parent_ref = spawn(parent_id.clone(), TestActor::new());
        let child_id = Uuid::new_v4();
        let child_ref = parent_ref.spawn(child_id.clone(), TestActor::new());
        parent_ref.clone().stop().map_err(|_| ()).map(move |_| {
            assert!(system.get_actor(&child_id).is_none());
            assert!(!child_ref.exists());
            assert!(system.get_actor(&parent_id).is_none());
            assert!(!parent_ref.exists());
        })
    })
}
