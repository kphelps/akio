use super::{ActorRef, ActorSystem};
use futures::future::Future;
use std::any::Any;
use std::cell::RefCell;
use tokio_core::reactor::Handle;

pub struct ThreadContext {
    pub handle: Handle,
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

pub fn execute<F>(f: F)
where
    F: Future<Item = (), Error = ()> + Send + 'static,
{
    let copied = with(ActorContext::clone);
    handle().spawn_fn(move || {
        set_current_actor(copied);
        f
    })
}

task_local! {
    static CURRENT_ACTOR: RefCell<Option<ActorContext>> = RefCell::new(None)
}

pub fn with_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut ActorContext) -> R,
{
    CURRENT_ACTOR.with(|ctx| f(ctx.borrow_mut().as_mut().unwrap()))
}

pub fn with<F, R>(f: F) -> R
where
    F: FnOnce(&ActorContext) -> R,
{
    CURRENT_ACTOR.with(|ctx| f(ctx.borrow().as_ref().unwrap()))
}

#[derive(Clone)]
pub struct ActorContext {
    pub system: ActorSystem,
}
