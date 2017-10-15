use super::{context, ActorRef};
use futures::Future;
use std::any::Any;
use uuid::Uuid;

enum ActorResponse<T> {
    Normal(T),
    Async(Box<Future<Item = T, Error = ()>>)
}

pub trait MessageHandler<T> {
    type Response;

    fn handle(&mut self, message: T) -> Self::Response;
}

pub trait Actor: 'static {
    fn handle_message<T>(&mut self, message: T)
        -> <Self as MessageHandler<T>>::Response
        where Self: MessageHandler<T>
    {
        self.handle(message)
    }

    fn on_start(&mut self) {}

    fn on_stop(&mut self) {}

    fn id(&self) -> Uuid {
        context::with(|ctx| ctx.self_ref.id())
    }
}
