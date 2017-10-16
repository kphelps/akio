use super::{context, Actor, ActorCellHandle, ActorSystem};
#[cfg(target_os = "linux")]
use core_affinity;
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
#[cfg(not(target_os = "linux"))]
use num_cpus;
use rand;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tokio_core::reactor::{Core, Remote};

lazy_static! {
    static ref START: Instant = Instant::now();
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
}


enum ThreadMessage {
    ProcessActor(Box<ActorProcessor>),
    Stop(),
}

trait ActorProcessor: Send + 'static {
    fn process(&self) -> usize;
}

impl<T> ActorProcessor for ActorCellHandle<T>
where
    T: Actor + Send,
{
    fn process(&self) -> usize {
        let n = self.process_messages(10);
        self.set_idle_or_dispatch();
        n
    }
}

struct ThreadHandle {
    sender: mpsc::Sender<ThreadMessage>,
    handle: thread::JoinHandle<()>,
    remote: Remote,
}

impl ThreadHandle {
    pub fn join(self) {
        self.send(ThreadMessage::Stop());
        self.handle.join().expect("Shutdown failed");
    }

    pub fn send(&self, message: ThreadMessage) {
        let f = self.sender
            .clone()
            .send(message)
            .map(|_| ())
            .map_err(|_| ());
        if let Some(handle) = context::maybe_handle() {
            handle.execute(f).unwrap();
        } else {
            self.remote.execute(f).unwrap();
        }
    }
}

pub(crate) struct Dispatcher {
    handles: Vec<ThreadHandle>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    pub fn start(&mut self, system: ActorSystem) {
        let _ = *START;
        self.handles = self.create_threads(system);
    }

    pub fn join(&mut self) {
        let handles = ::std::mem::replace(&mut self.handles, Vec::new());
        handles.into_iter().for_each(ThreadHandle::join);
    }

    pub fn dispatch<T>(&self, actor: ActorCellHandle<T>)
    where
        T: Actor,
    {
        let thread_handle = self.next_thread();
        thread_handle.send(ThreadMessage::ProcessActor(Box::new(actor)));
    }

    #[cfg(target_os = "linux")]
    fn create_threads(&self, system: ActorSystem) -> Vec<ThreadHandle> {
        core_affinity::get_core_ids()
            .unwrap()
            .into_iter()
            .map(|core_id| self.create_thread(core_id, system.clone()))
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(target_os = "linux")]
    fn create_thread(&self, core_id: core_affinity::CoreId, system: ActorSystem) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let (remote, handle) = DispatcherThread::new(system, receiver).run_with_affinity(core_id);
        ThreadHandle {
            sender: sender,
            handle: handle,
            remote: remote,
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn create_threads(&self, system: ActorSystem) -> Vec<ThreadHandle> {
        iter::repeat(())
            .take(num_cpus::get())
            .map(|_| self.create_thread(system))
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(not(target_os = "linux"))]
    fn create_thread(&self, system: ActorSystem) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let (remote, handle) = DispatcherThread::new(system, receiver).run();
        ThreadHandle {
            sender: sender,
            handle: handle,
            remote: remote,
        }
    }

    fn next_thread(&self) -> &ThreadHandle {
        let mut rng = rand::thread_rng();
        let handle = rng.choose(&self.handles);
        handle.unwrap()
    }
}

struct DispatcherThread {
    receiver: mpsc::Receiver<ThreadMessage>,
    system: ActorSystem,
}

impl DispatcherThread {
    pub fn new(system: ActorSystem, receiver: mpsc::Receiver<ThreadMessage>) -> Self {
        Self {
            receiver: receiver,
            system: system,
        }
    }

    pub fn run(self) -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            let stream = self.receiver.for_each(|message| handle_message(message));
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext {
                handle: handle,
                system: self.system,
            });
            let _ = core.run(stream);
        });
        // Need to extract the Remote from the new thread
        loop {
            if arc_remote.lock().unwrap().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let remote = arc_remote.lock().unwrap().as_ref().unwrap().clone();
        (remote, handle)
    }

    #[cfg(target_os = "linux")]
    pub fn run_with_affinity(
        self,
        core_id: core_affinity::CoreId,
    ) -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            core_affinity::set_for_current(core_id);
            let stream = self.receiver.for_each(|message| handle_message(message));
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext {
                handle: handle,
                system: self.system,
            });
            let _ = core.run(stream);
        });
        // Need to extract the Remote from the new thread
        loop {
            if arc_remote.lock().unwrap().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let remote = arc_remote.lock().unwrap().as_ref().unwrap().clone();
        (remote, handle)
    }
}

fn handle_message(message: ThreadMessage) -> Result<(), ()> {
    match message {
        ThreadMessage::ProcessActor(processor) => {
            let n = processor.process();
            count(n);
            Ok(())
        }
        ThreadMessage::Stop() => Err(()),
    }
}

fn count(n: usize) {
    let count = COUNTER.fetch_add(n, Ordering::SeqCst) + n;
    if (count - n) % 10000000 > count % 10000000 {
        let dt = (Instant::now() - *START).as_secs() as usize;
        if dt > 0 {
            let rate = count / dt;
            println!("Dispatch {} ({}/s)", count, rate);
        }
    }
}
