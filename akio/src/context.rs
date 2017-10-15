use super::{ActorRef, ActorSystem};
use futures::future::Future;
use std::cell::RefCell;
use tokio_core::reactor::Handle;

pub struct ThreadContext {
    pub handle: Handle,
    pub system: ActorSystem,
}

thread_local! {
    static CURRENT_THREAD: RefCell<Option<ThreadContext>> = RefCell::new(None)
}

pub fn set_thread_context(context: ThreadContext) {
    CURRENT_THREAD.with(|ctx| *ctx.borrow_mut() = Some(context))
}

pub fn handle() -> Handle {
    CURRENT_THREAD.with(|ctx| ctx.borrow().as_ref().unwrap().handle.clone())
}

pub fn system() -> ActorSystem {
    CURRENT_THREAD.with(|ctx| ctx.borrow().as_ref().unwrap().system.clone())
}

pub fn execute<F>(f: F)
where
    F: Future<Item = (), Error = ()> + Send + 'static,
{
    handle().spawn(f)
}
