extern crate akio_net;
extern crate env_logger;
extern crate futures;
extern crate tokio_core;

use akio_net::*;
use futures::future;
use tokio_core::reactor::Core;


pub fn main() {
    env_logger::init().unwrap();
    let mut core = Core::new().unwrap();
    let node = RemoteNode::new(&core.handle(), &"127.0.0.1:6667".parse().unwrap()).unwrap();
    node.connect(&"127.0.0.1:6666".parse().unwrap());
    core.run(future::empty::<(), ()>()).unwrap();
}
