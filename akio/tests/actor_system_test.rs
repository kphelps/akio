extern crate akio;

use akio::prelude::*;

#[test]
fn test_start_stop_actor_system() {
    let system = ActorSystem::new();
    system.stop();
}
