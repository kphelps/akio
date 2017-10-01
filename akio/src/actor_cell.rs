use super::{ActorContext, ActorRef, ActorSystem, BaseActor, context, Mailbox, MailboxMessage};
use futures::stream;
use futures::prelude::*;
use parking_lot::ReentrantMutex;
use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
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
    inner: Arc<ReentrantMutex<RefCell<ActorCellInner>>>,
}

pub struct ActorCellInner {
    id: Uuid,
    mailbox: Mailbox,
    status: ActorStatus,
    actor: Box<BaseActor>,
    system: ActorSystem,
}

impl ActorCell {
    pub fn new<A>(system: ActorSystem, id: Uuid, actor: A) -> Self
        where A: BaseActor + 'static
    {
        let inner = ActorCellInner {
            id: id,
            mailbox: Mailbox::new(),
            status: ActorStatus::Idle,
            actor: Box::new(actor),
            system: system,
        };
        let p_inner = Arc::new(ReentrantMutex::new(RefCell::new(inner)));
        let cell = Self { inner: p_inner };
        cell
    }

    pub fn id(&self) -> Uuid {
        self.with_inner(|inner| inner.id.clone())
    }

    pub fn process_messages(&mut self,
                            max_count: usize)
                            -> Box<Future<Item = (), Error = ()> + 'static> {
        let self_ref = ActorRef::new(self.clone());
        let locked_inner = self.inner.lock();
        let mut inner = locked_inner.borrow_mut();
        inner.set_current_actor(self_ref);
        let message_batch = inner.next_batch_to_process(max_count);

        let futures = message_batch
            .into_iter()
            .map(|message| inner.process_message(message));
        Box::new(stream::futures_ordered(futures).collect().map(|_| ()))
    }

    pub fn enqueue_message(&self, message: Box<Any + Send>, sender: ActorRef) {
        let me = self.clone();
        self.with_inner_mut(|inner| {
                                inner.enqueue_message(message, sender);
                                inner.dispatch(me);
                            });
    }

    pub fn set_idle_or_dispatch(&mut self) {
        let me = self.clone();
        self.with_inner_mut(|inner| if inner.mailbox.is_empty() {
                                inner.set_status(ActorStatus::Idle);
                            } else {
                                inner.system.dispatch(me)
                            });
    }

    pub fn set_idle(&mut self) {
        println!("Idle");
        self.set_status(ActorStatus::Idle);
    }

    pub fn set_scheduled(&mut self) -> bool {
        self.with_inner_mut(|inner| {
                                if inner.status.is_scheduled() {
                                    return true;
                                };
                                inner.set_status(ActorStatus::Scheduled);
                                println!("Scheduled");
                                false
                            })
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
        let locked = self.inner.lock();
        let borrowed = locked.borrow();
        f(&borrowed)
    }

    fn with_inner_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut ActorCellInner) -> R
    {
        let locked = self.inner.lock();
        let mut borrowed = locked.borrow_mut();
        f(&mut borrowed)
    }
}

impl ActorCellInner {
    pub fn set_current_actor(&self, self_ref: ActorRef) {
        context::set_current_actor(ActorContext::new(self_ref, self.system.clone()))
    }

    pub fn dispatch(&mut self, cell: ActorCell) {
        if self.status.is_scheduled() {
            println!("Ignored");
            return;
        }
        self.system.dispatch(cell);
    }

    fn process_message(&mut self,
                       message: MailboxMessage)
                       -> Box<Future<Item = (), Error = ()> + 'static> {
        match message {
            MailboxMessage::User(inner, sender) => {
                context::set_sender(sender);
                self.actor.handle_any(inner)
            }
        }
    }

    pub fn enqueue_message(&mut self, message: Box<Any + Send>, sender: ActorRef) {
        self.mailbox.push(message, sender);
    }

    pub fn next_to_process(&mut self) -> Option<MailboxMessage> {
        self.mailbox.pop()
    }

    pub fn next_batch_to_process(&mut self, count: usize) -> VecDeque<MailboxMessage> {
        let mut v = VecDeque::with_capacity(count);
        for _ in 0..count {
            match self.next_to_process() {
                Some(message) => v.push_back(message),
                None => return v,
            }
        }
        v
    }

    pub fn set_status(&mut self, status: ActorStatus) {
        self.status = status;
    }
}
