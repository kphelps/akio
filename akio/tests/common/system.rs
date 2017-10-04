use super::futures::future;
use super::futures::sync::oneshot;
use akio::prelude::*;

pub fn with_actor_system_async<F, R>(f: F) -> R
where
    F: FnOnce(ActorSystem)
        -> Box<Future<Item = R, Error = ()> + Send + 'static>
        + Send
        + 'static,
    R: Send + 'static,
{
    let mut system = ActorSystem::new();
    let system_clone = system.clone();
    let (sender, receiver) = oneshot::channel();
    system.on_startup(move || {
        let fut = f(system_clone.clone()).map(|f_result| sender.send(f_result).ok().unwrap());
        context::execute(fut);
    });
    let result = receiver.wait();
    system.stop();
    result.unwrap()
}

pub fn with_actor_system<F, R>(f: F) -> R
where
    F: FnOnce(ActorSystem) -> R + Send + 'static,
    R: Send + 'static,
{
    with_actor_system_async(|system| {
        let result = f(system);
        Box::new(future::ok(result))
    })
}
