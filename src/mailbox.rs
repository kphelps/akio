use std::collections::VecDeque;

pub enum MailboxMessage<T> {
    User(T),
}

pub struct Mailbox<T> {
    messages: VecDeque<MailboxMessage<T>>,
}

impl<T> Mailbox<T> {
    pub fn new() -> Self {
        Self { messages: VecDeque::new() }
    }

    pub fn push(&mut self, message: T) {
        self.messages.push_back(MailboxMessage::User(message))
    }

    pub fn pop(&mut self) -> Option<MailboxMessage<T>> {
        self.messages.pop_front()
    }
}
