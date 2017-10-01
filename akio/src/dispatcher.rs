#[cfg(target_os = "linux")]
use core_affinity;

#[cfg(not(target_os = "linux"))]
use num_cpus;

use super::{ActorCellHandle, context};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use rand;
use rand::Rng;
use std::iter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio_core::reactor::{Core, Remote};

lazy_static! {
    static ref START: Instant = Instant::now();
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
}


enum ThreadMessage {
    ProcessActor(ActorCellHandle),
}

struct ThreadHandle {
    sender: mpsc::Sender<ThreadMessage>,
    handle: thread::JoinHandle<()>,
    remote: Remote,
}

impl ThreadHandle {
    pub fn join(self) {
        self.handle.join().expect("Shutdown failed")
    }
}

pub struct Dispatcher {
    handles: Vec<ThreadHandle>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self { handles: Vec::new() }
    }

    pub fn start(&mut self) {
        println!("Starting: {:?}", *START);
        self.handles = self.create_threads();
    }

    pub fn join(self) {
        self.handles.into_iter().for_each(ThreadHandle::join)
    }

    pub fn dispatch(&self, actor: ActorCellHandle) {
        let thread_handle = self.next_thread();

        let f = thread_handle
            .sender
            .clone()
            .send(ThreadMessage::ProcessActor(actor))
            .map(|_| ())
            .map_err(|_| ());
        thread_handle.remote.execute(f).unwrap();
    }

    #[cfg(target_os = "linux")]
    fn create_threads(&self) -> Vec<ThreadHandle> {
        core_affinity::get_core_ids()
            .unwrap()
            .into_iter()
            .map(|core_id| self.create_thread(core_id))
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(target_os = "linux")]
    fn create_thread(&self, core_id: core_affinity::CoreId) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let (remote, handle) = DispatcherThread::new(receiver).run_with_affinity(core_id);
        ThreadHandle {
            sender: sender,
            handle: handle,
            remote: remote,
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn create_threads(&self) -> Vec<ThreadHandle> {
        iter::repeat(())
            .take(num_cpus::get())
            .map(|_| self.create_thread())
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(not(target_os = "linux"))]
    fn create_thread(&self) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let (remote, handle) = DispatcherThread::new(receiver).run();
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
}

impl DispatcherThread {
    pub fn new(receiver: mpsc::Receiver<ThreadMessage>) -> Self {
        Self { receiver: receiver }
    }

    pub fn run(self) -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            println!("Starting thread: {:?}", thread::current().id());
            let stream = self.receiver
                .for_each(|message| {
                              handle_message(message);
                              Ok(())
                          });
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext { handle: handle });
            core.run(stream).expect("Dispatcher failure");
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
    pub fn run_with_affinity(self,
                             core_id: core_affinity::CoreId)
                             -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            println!("Starting thread: {:?}", thread::current().id());
            core_affinity::set_for_current(core_id);
            let stream = self.receiver
                .for_each(|message| {
                              handle_message(message);
                              Ok(())
                          });
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext { handle: handle });
            core.run(stream).expect("Dispatcher failure");
        });
        // Need to extract the Remote from the new thread
        loop {
            if arc_remote.lock().unwrap().is_some() {
                break;
            }
            thread::sleep_ms(10);
        }
        let remote = arc_remote.lock().unwrap().as_ref().unwrap().clone();
        (remote, handle)
    }
}

fn handle_message(message: ThreadMessage) {
    match message {
        ThreadMessage::ProcessActor(actor_cell) => {
            let n = actor_cell.process_messages(10);
            count(n);
            actor_cell.set_idle_or_dispatch();
        }
    }
}

fn count(n: usize) {
    let count = COUNTER.fetch_add(n, Ordering::SeqCst) + n;
    if count % 100000 == 0 {
        let dt = (Instant::now() - *START).as_secs() as usize;
        if dt > 0 {
            let rate = count / dt;
            println!("Dispatch {} ({}/s)", count, rate);
        }
        if count > 100000000 {
            ::std::process::exit(0);
        }
    }
}
