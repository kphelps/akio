use super::{Actor, ActorResponse, MessageHandler};
use futures::sync::oneshot;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub enum SystemMessage {
    Stop(oneshot::Sender<()>),
}

pub(crate) enum MailboxMessage<A> {
    User(UserMessageWrapper<A>),
    System(SystemMessage),
}

pub(crate) struct UserMessageWrapper<A>(Box<UserMessage<A>>);

impl<A> UserMessageWrapper<A>
where
    A: Actor,
{
    pub fn make<M>(message: M, promise: Option<oneshot::Sender<ActorResponse<A::Response>>>) -> Self
    where
        M: Send + 'static,
        A: MessageHandler<M>,
    {
        UserMessageWrapper(Box::new(LocalUserMessage::new(message, promise)))
    }

    pub fn handle(&mut self, actor: &mut A) {
        self.0.handle(actor)
    }
}

trait UserMessage<A>: Send {
    fn handle(&mut self, actor: &mut A);
}

struct LocalUserMessage<A, M>
where
    A: MessageHandler<M>,
{
    message: Option<M>,
    promise: Option<oneshot::Sender<ActorResponse<A::Response>>>,
}

impl<A, M> LocalUserMessage<A, M>
where
    A: MessageHandler<M>,
{
    pub fn new(message: M, promise: Option<oneshot::Sender<ActorResponse<A::Response>>>) -> Self {
        Self {
            message: Some(message),
            promise: promise,
        }
    }
}

impl<A, M> UserMessage<A> for LocalUserMessage<A, M>
where
    A: Actor + MessageHandler<M>,
    M: Send,
{
    fn handle(&mut self, actor: &mut A) {
        if let Some(message) = self.message.take() {
            let response = actor.handle_message(message);
            self.promise.take().map(|promise| promise.send(response));
        }
    }
}

pub(crate) struct Mailbox<A> {
    messages: VecDeque<MailboxMessage<A>>,
}

impl<A> Mailbox<A>
where
    A: Actor,
{
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn push<M>(
        &mut self,
        message: M,
        promise: Option<oneshot::Sender<ActorResponse<A::Response>>>,
    ) where
        A: MessageHandler<M>,
        M: Send + 'static,
    {
        self.messages.push_back(MailboxMessage::User(
            UserMessageWrapper::make(message, promise),
        ))
    }

    pub fn push_system_message(&mut self, system_message: SystemMessage) {
        self.messages
            .push_back(MailboxMessage::System(system_message))
    }

    pub fn pop(&mut self) -> Option<MailboxMessage<A>> {
        self.messages.pop_front()
    }
}
