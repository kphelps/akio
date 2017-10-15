use super::{ActorRef, context, create_actor};
use futures::{Async, IntoFuture, Future, future, Poll};
use uuid::Uuid;

pub enum ActorResponse<T> {
    Normal(Option<T>),
    Async(Box<Future<Item = T, Error = ()> + Send>)
}

impl<T> Future for ActorResponse<T>
{
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            ActorResponse::Normal(ref mut value) =>
                Ok(Async::Ready(value.take().unwrap())),
            ActorResponse::Async(ref mut f) => f.poll(),
        }
    }
}

pub trait MessageHandler<T> {
    type Response: Send;

    fn handle(&mut self, message: T) -> ActorResponse<Self::Response>;
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
        -> ActorResponse<<Self as MessageHandler<T>>::Response>
        where Self: MessageHandler<T>
    {
        self.handle(message)
    }

    fn on_start(&mut self) {}

    fn on_stop(&mut self) {}

    fn start(self) -> ActorRef<Self> {
        create_actor(context::system(), Uuid::new_v4(), self)
    }

    fn done(&self) -> ActorResponse<()> {
        self.respond(())
    }

    fn respond<T>(&self, v: T) -> ActorResponse<T> {
        ActorResponse::Normal(Some(v))
    }

    fn respond_fut<F, T>(&self, v: F) -> ActorResponse<T>
        where F: IntoFuture<Item = T, Error = ()> + 'static,
              F::Future: Send
    {
        ActorResponse::Async(Box::new(v.into_future()))
    }
}
