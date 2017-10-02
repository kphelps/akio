use super::{ActorChildren, ActorContext, ActorRef, ActorSystem, BaseActor, context, create_actor,
            Mailbox, MailboxMessage};
use parking_lot::Mutex;
use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum ActorStatus {
    Idle,
    Scheduled,
}

impl ActorStatus {
    pub fn is_scheduled(&self) -> bool {
        match self {
            &ActorStatus::Scheduled => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct ActorCellHandle {
    cell: Arc<ActorCell>,
}

impl ActorCellHandle {
    pub fn process_messages(&self, max_count: usize) -> usize {
        let self_ref = ActorRef::new(self.clone());
        self.cell.process_messages(self_ref, max_count)
    }

    pub fn enqueue_message(&self, message: Box<Any + Send>, sender: ActorRef) {
        self.cell.enqueue_message(self.clone(), message, sender)
    }

    pub fn set_idle_or_dispatch(&self) {
        self.cell.set_idle_or_dispatch(self.clone())
    }

    pub fn spawn<A>(&self, id: Uuid, actor: A) -> ActorRef
        where A: BaseActor + 'static
    {
        self.cell.spawn(id, actor)
    }

    pub fn with_children<F, R>(&self, f: F) -> R
        where F: FnOnce(&ActorChildren) -> R
    {
        f(&self.cell.children.lock())
    }
}

pub struct ActorCell {
    inner: Mutex<ActorCellInner>,
    mailbox: Mutex<Mailbox>,
    children: Mutex<ActorChildren>,
    system: ActorSystem,
}

pub struct ActorCellInner {
    id: Uuid,
    status: ActorStatus,
    actor: Box<BaseActor>,
    system: ActorSystem,
}

impl ActorCell {
    pub fn new<A>(system: ActorSystem, id: Uuid, actor: A) -> ActorCellHandle
        where A: BaseActor + 'static
    {
        let mailbox = Mutex::new(Mailbox::new());
        let inner = ActorCellInner {
            id: id,
            status: ActorStatus::Idle,
            actor: Box::new(actor),
            system: system.clone(),
        };
        let p_inner = Mutex::new(inner);
        let cell = Self {
            inner: p_inner,
            mailbox: mailbox,
            children: Mutex::new(ActorChildren::new()),
            system: system.clone(),
        };
        ActorCellHandle { cell: Arc::new(cell) }
    }

    pub fn process_messages(&self, self_ref: ActorRef, max_count: usize) -> usize {
        let message_batch = self.next_batch_to_process(max_count);
        let count = message_batch.len();
        let mut inner = self.inner.lock();
        inner.set_current_actor(self_ref);
        message_batch
            .into_iter()
            .for_each(|message| inner.process_message(message));
        count
    }

    pub fn next_batch_to_process(&self, count: usize) -> VecDeque<MailboxMessage> {
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

    pub fn enqueue_message(&self,
                           me: ActorCellHandle,
                           message: Box<Any + Send>,
                           sender: ActorRef) {
        self.mailbox.lock().push(message, sender);
        self.try_with_inner_mut(|inner| { inner.dispatch(me); });
    }

    pub fn set_idle_or_dispatch(&self, me: ActorCellHandle) {
        let mailbox = self.mailbox.lock();
        self.with_inner_mut(|inner| if mailbox.is_empty() {
                                inner.set_status(ActorStatus::Idle);
                            } else {
                                inner.system.dispatch(me)
                            });
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

    pub fn spawn<A>(&self, id: Uuid, actor: A) -> ActorRef
        where A: BaseActor + 'static
    {
        let actor_ref = create_actor(&self.system, id, actor);
        self.children.lock().insert(id, &actor_ref);
        actor_ref
    }
}

impl ActorCellInner {
    pub fn set_current_actor(&self, self_ref: ActorRef) {
        context::set_current_actor(ActorContext::new(self_ref, self.system.clone()))
    }

    pub fn dispatch(&mut self, cell: ActorCellHandle) {
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
