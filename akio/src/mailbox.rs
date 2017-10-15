use super::ActorRef;
use futures::sync::oneshot;
use std::any::Any;
use std::collections::VecDeque;

pub enum SystemMessage {
    Stop(oneshot::Sender<()>),
}

pub enum MailboxMessage {
    User(Box<Any + Send>),
    System(SystemMessage),
}

pub struct Mailbox {
    messages: VecDeque<MailboxMessage>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn push(&mut self, message: Box<Any + Send>) {
        self.messages
            .push_back(MailboxMessage::User(message, sender))
    }

    pub fn push_system_message(&mut self, system_message: SystemMessage) {
        self.messages
            .push_back(MailboxMessage::System(system_message))
    }

    pub fn pop(&mut self) -> Option<MailboxMessage> {
        self.messages.pop_front()
    }
}
