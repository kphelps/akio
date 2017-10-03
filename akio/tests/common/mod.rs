extern crate futures;

mod system;
mod test_actor;

pub use self::system::with_actor_system;
pub use self::system::with_actor_system_async;
pub use self::test_actor::*;
