use super::{ActorChildren, ActorRef, context};
use std::any::Any;
use uuid::Uuid;

pub trait BaseActor: Send {
    fn handle_any(&mut self, message: Box<Any + Send>);

    fn on_start(&mut self);

    fn on_stop(&mut self);
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, message: Self::Message);

    fn on_start_impl(&mut self) {}

    fn on_stop_impl(&mut self) {}

    fn with_children<F, R>(&self, f: F) -> R
        where F: FnOnce(&ActorChildren) -> R
    {
        context::with(|ctx| ctx.self_ref.with_children(f))
    }

    fn sender_ref(&self) -> ActorRef {
        context::with(|ctx| ctx.sender.clone())
    }

    fn sender<T: TypedActor>(&self) -> T::RefType {
        context::with(|ctx| T::from_ref(ctx.sender.clone()))
    }
}

impl<T> BaseActor for T
    where T: Actor + Send
{
    fn handle_any(&mut self, any_message: Box<Any + Send>) {
        match any_message.downcast() {
            Ok(message) => self.handle_message(*message),
            _ => {
                println!("Unhandled message");
            }
        }
    }

    fn on_start(&mut self) {
        self.on_start_impl()
    }

    fn on_stop(&mut self) {
        self.on_stop_impl()
    }
}

pub trait TypedActor {
    type RefType;

    fn from_ref(actor_ref: ActorRef) -> Self::RefType;
}

pub fn spawn<T>(id: Uuid, actor: T) -> T::RefType
    where T: Actor + TypedActor + Send + 'static
{
    context::with_mut(|ctx| {
                          let actor_ref = ctx.self_ref.spawn(id, actor);
                          T::from_ref(actor_ref)
                      })
}
