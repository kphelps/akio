use super::{ActorContext, ActorRef, ActorSystem, BaseActor, context, Mailbox, MailboxMessage};
use futures::stream;
use futures::prelude::*;
use parking_lot::Mutex;
use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum ActorStatus {
    Idle,
    Scheduled,
}

impl ActorStatus {
    pub fn is_idle(&self) -> bool {
        match self {
            &ActorStatus::Idle => true,
            _ => false,
        }
    }

    pub fn is_scheduled(&self) -> bool {
        match self {
            &ActorStatus::Scheduled => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct ActorCell {
    inner: Arc<Mutex<ActorCellInner>>,
    mailbox: Arc<Mutex<Mailbox>>,
}

pub struct ActorCellInner {
    id: Uuid,
    status: ActorStatus,
    actor: Box<BaseActor>,
    system: ActorSystem,
}

impl ActorCell {
    pub fn new<A>(system: ActorSystem, id: Uuid, actor: A) -> Self
        where A: BaseActor + 'static
    {
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        let inner = ActorCellInner {
            id: id,
            status: ActorStatus::Idle,
            actor: Box::new(actor),
            system: system,
        };
        let p_inner = Arc::new(Mutex::new(inner));
        let cell = Self {
            inner: p_inner,
            mailbox: mailbox,
        };
        cell
    }

    pub fn id(&self) -> Uuid {
        self.with_inner(|inner| inner.id.clone())
    }

    pub fn process_messages(&mut self, max_count: usize) {
        let self_ref = ActorRef::new(self.clone());
        let message_batch = self.next_batch_to_process(max_count);
        let mut inner = self.inner.lock();
        inner.set_current_actor(self_ref);

        message_batch
            .into_iter()
            .for_each(|message| inner.process_message(message));
    }

    pub fn next_batch_to_process(&mut self, count: usize) -> VecDeque<MailboxMessage> {
        let mut mailbox = self.mailbox.lock();
        let mut v = VecDeque::with_capacity(count);
        for _ in 0..count {
            match mailbox.pop() {
                Some(message) => v.push_back(message),
                None => return v,
            }
        }
        v
    }

    pub fn enqueue_message(&self, message: Box<Any + Send>, sender: ActorRef) {
        let me = self.clone();
        self.mailbox.lock().push(message, sender);
        self.try_with_inner_mut(|inner| { inner.dispatch(me); });
    }

    pub fn set_idle_or_dispatch(&mut self) {
        let me = self.clone();
        let mailbox = self.mailbox.lock();
        self.with_inner_mut(|inner| if mailbox.is_empty() {
                                inner.set_status(ActorStatus::Idle);
                            } else {
                                inner.system.dispatch(me)
                            });
    }

    pub fn set_idle(&mut self) {
        self.set_status(ActorStatus::Idle);
    }

    fn set_status(&mut self, status: ActorStatus) {
        self.with_inner_mut(|inner| inner.set_status(status))
    }

    pub fn is_scheduled(&self) -> bool {
        match self.get_status() {
            ActorStatus::Scheduled => true,
            _ => false,
        }
    }

    pub fn get_status(&self) -> ActorStatus {
        self.with_inner(|inner| inner.status)
    }

    fn with_inner<F, R>(&self, f: F) -> R
        where F: FnOnce(&ActorCellInner) -> R
    {
        let mut inner = self.inner.lock();
        let x = f(&inner);
        x
    }

    fn with_inner_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut ActorCellInner) -> R
    {
        let mut inner = self.inner.lock();
        let x = f(&mut inner);
        x
    }

    fn try_with_inner_mut<F, R>(&self, f: F) -> Option<R>
        where F: FnOnce(&mut ActorCellInner) -> R
    {
        self.inner.try_lock().map(|mut inner| f(&mut inner))
    }
}

impl ActorCellInner {
    pub fn set_current_actor(&self, self_ref: ActorRef) {
        context::set_current_actor(ActorContext::new(self_ref, self.system.clone()))
    }

    pub fn dispatch(&mut self, cell: ActorCell) {
        if self.status.is_scheduled() {
            return;
        }
        self.set_status(ActorStatus::Scheduled);
        self.system.dispatch(cell);
    }

    fn process_message(&mut self, message: MailboxMessage) {
        match message {
            MailboxMessage::User(inner, sender) => {
                context::set_sender(sender);
                self.actor.handle_any(inner)
            }
        }
    }

    pub fn set_status(&mut self, status: ActorStatus) {
        self.status = status;
    }
}
