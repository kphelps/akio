extern crate akio_net;
extern crate futures;
extern crate tokio_core;

use akio_net::*;
use futures::future;
use tokio_core::reactor::Core;


pub fn main() {
    let mut core = Core::new().unwrap();
    let node = RemoteNode::new(&core.handle(), 6666).unwrap();
    core.run(future::empty::<(), ()>());
}
