use super::{ActorRef, ActorSystem};
use futures::future::{Executor, Future, ExecuteError};
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

pub fn execute<F>(f: F) -> Result<(), ExecuteError<F>>
    where F: Future<Item = (), Error = ()> + Send + 'static
{
    handle().execute(f)
}

task_local! {
    static CURRENT_ACTOR: RefCell<Option<ActorContext>> = RefCell::new(None)
}

pub fn with_mut<F, R>(f: F) -> R
    where F: FnOnce(&mut ActorContext) -> R
{
    CURRENT_ACTOR.with(|ctx| f(ctx.borrow_mut().as_mut().unwrap()))
}

pub fn with<F, R>(f: F) -> R
    where F: FnOnce(&ActorContext) -> R
{
    CURRENT_ACTOR.with(|ctx| f(ctx.borrow().as_ref().unwrap()))
}

pub fn set_current_actor(context: ActorContext) {
    CURRENT_ACTOR.with(|ctx| *ctx.borrow_mut() = Some(context))
}

pub fn set_sender(sender: ActorRef) {
    with_mut(|ctx| ctx.sender = sender)
}

pub fn get_current_sender() -> ActorRef {
    with(|ctx| ctx.sender.clone())
}

#[derive(Clone)]
pub struct ActorContext {
    pub self_ref: ActorRef,
    pub sender: ActorRef,
    pub system: ActorSystem,
}

impl ActorContext {
    pub fn new(self_ref: ActorRef, system: ActorSystem) -> Self {
        Self {
            self_ref: self_ref.clone(),
            sender: self_ref,
            system: system,
        }
    }

    pub fn send<T: Any + Send>(&self, message: T, target: &ActorRef) {
        target.send(message, &self.self_ref)
    }

    pub fn reply<T: Any + Send>(&self, message: T) {
        self.send(message, &self.sender)
    }
}
