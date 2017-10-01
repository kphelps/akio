use super::ActorRef;
use std::any::Any;
use std::collections::VecDeque;

pub enum MailboxMessage {
    User(Box<Any + Send>, ActorRef),
}

pub struct Mailbox {
    messages: VecDeque<MailboxMessage>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self { messages: VecDeque::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn push(&mut self, message: Box<Any + Send>, sender: ActorRef) {
        self.messages
            .push_back(MailboxMessage::User(message, sender))
    }

    pub fn pop(&mut self) -> Option<MailboxMessage> {
        self.messages.pop_front()
    }
}
