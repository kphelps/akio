use super::{ActorRef, context, create_actor};
use futures::Future;
use uuid::Uuid;

enum ActorResponse<T> {
    Normal(T),
    Async(Box<Future<Item = T, Error = ()>>)
}

pub trait MessageHandler<T> {
    type Response;

    fn handle(&mut self, message: T) -> Self::Response;
}

pub struct ActorContext<A> {
    self_ref: ActorRef<A>,
}

impl<A> ActorContext<A>
    where A: Actor
{
    pub fn id(&self) -> Uuid {
        self.self_ref.id()
    }
}

pub trait Actor: Sized + Send + 'static {
    fn handle_message<T>(&mut self, message: T)
        -> <Self as MessageHandler<T>>::Response
        where Self: MessageHandler<T>
    {
        self.handle(message)
    }

    fn on_start(&mut self) {}

    fn on_stop(&mut self) {}

    fn start(self) -> ActorRef<Self> {
        create_actor(context::system(), Uuid::new_v4(), self)
    }
}
