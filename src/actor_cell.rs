use super::{Actor, ActorSystemHandle, Mailbox, MailboxMessage};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use uuid::Uuid;

pub trait BaseActorCell {
    fn process_message(&mut self);
}

pub struct ActorCell<T> {
    inner: Rc<RefCell<ActorCellInner<T>>>,
}

pub struct ActorCellInner<T> {
    id: Uuid,
    mailbox: Mailbox<T>,
    actor: Box<Actor<T>>,
    system_handle: ActorSystemHandle,
}

#[derive(Clone)]
pub struct ActorCellHandle<T> {
    inner: Weak<RefCell<ActorCellInner<T>>>,
}

impl<T> ActorCell<T> {
    pub fn new<A>(id: Uuid, actor: A, system_handle: ActorSystemHandle) -> Self
        where A: Actor<T>,
              A: 'static
    {
        let inner = ActorCellInner::<T> {
            id: id,
            mailbox: Mailbox::new(),
            actor: Box::new(actor),
            system_handle: system_handle,
        };
        Self { inner: Rc::new(RefCell::new(inner)) }
    }

    pub fn handle(&self) -> ActorCellHandle<T> {
        ActorCellHandle::<T> { inner: Rc::downgrade(&self.inner) }
    }
}

impl<T> ActorCellInner<T> {
    pub fn enqueue_message(&mut self, message: T) {
        self.mailbox.push(message);
        self.system_handle.mailbox_ready(self.id);
    }

    pub fn process_message(&mut self) {
        match self.mailbox.pop().expect("mailbox is empty") {
            MailboxMessage::User(inner) => self.actor.handle_message(inner),
        }
    }
}

impl<T> BaseActorCell for ActorCell<T> {
    fn process_message(&mut self) {
        self.inner.borrow_mut().process_message();
    }
}

impl<T> ActorCellHandle<T> {
    pub fn enqueue_message(&mut self, message: T) {
        let inner = self.inner.upgrade().expect("Failed to enqueue");
        inner.deref().borrow_mut().enqueue_message(message);
    }
}
