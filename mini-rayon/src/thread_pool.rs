use std::thread;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use rand;
use num_cpus;
use once_cell::sync::Lazy;
use deque::{Worker, Stealer, Stolen};
use crate::job::Job;
use crate::latch::Latch;

struct RegistryState {
    threads_at_work: usize,
    injected_jobs: Vec<Job>,
}

impl RegistryState {
    fn new() -> RegistryState {
        RegistryState {
            threads_at_work: 0,
            injected_jobs: Vec::new(),
        }
    }
}

pub static THE_REGISTRY: Lazy<Arc<Registry>> = Lazy::new(|| {
    let registry = Registry::new();
    registry.wait_until_primed();
    registry
});

pub fn get_registry() -> &'static Arc<Registry> {
    &THE_REGISTRY
}

pub struct Registry {
    thread_infos: Vec<ThreadInfo>,
    state: Mutex<RegistryState>,
    work_available: Condvar,
    terminated: AtomicBool,
}

unsafe impl Send for Registry { }
unsafe impl Sync for Registry { }

impl Registry {
    pub fn new() -> Arc<Registry> {
        let num_threads = num_cpus::get();
        let registry = Arc::new(Registry {
            thread_infos: (0..num_threads).map(|_| ThreadInfo::new()).collect(),
            state: Mutex::new(RegistryState::new()),
            work_available: Condvar::new(),
            terminated: AtomicBool::new(false),
        });
        for index in 0..num_threads {
            let registry = registry.clone();
            thread::spawn(move || {
                main_loop(registry, index);
            });
        }
        registry
    }

    pub fn num_threads(&self) -> usize {
        self.thread_infos.len()
    }

    pub fn wait_until_primed(&self) {
        self.thread_infos.iter().for_each(|info| {
            info.primed.wait();
        });
    }

    pub fn terminate(&self) {
        self.terminated.store(true, Ordering::SeqCst);
    }

    pub fn inject(&self, injected_jobs: Vec<Job>) {
        let mut state = self.state.lock().unwrap();
        state.injected_jobs.extend(injected_jobs);
        self.work_available.notify_all();
    }

    fn start_working(&self) {
        let mut state = self.state.lock().unwrap();
        state.threads_at_work += 1;
        self.work_available.notify_all();
    }

    fn wait_for_work(&self, was_active: bool) -> Option<Job> {
        let mut state = self.state.lock().unwrap();
        if was_active {
            state.threads_at_work -= 1;
        }
        loop {
            if let Some(job) = state.injected_jobs.pop() {
                return Some(job);
            }
            if state.threads_at_work > 0 {
                return None;
            }
            state = self.work_available.wait(state).unwrap();
        }
    }
}

struct ThreadInfo {
    primed: Latch,
    worker: Worker<Job>,
    stealer: Stealer<Job>,
}

impl ThreadInfo {
    fn new() -> ThreadInfo {
        let (worker, stealer) = deque::new();
        ThreadInfo {
            primed: Latch::new(),
            worker,
            stealer,
        }
    }
}

thread_local! {
    static WORKER_THREAD_STATE: RefCell<Option<Arc<WorkerThread>>> = RefCell::new(None);
}

pub struct WorkerThread {
    registry: Arc<Registry>,
    index: usize,
}

impl WorkerThread {
    pub fn current() -> Option<Arc<WorkerThread>> {
        WORKER_THREAD_STATE.with(|worker_thread_state| {
            worker_thread_state.borrow().as_ref().cloned()
        })
    }

    pub fn set_current(current: Arc<WorkerThread>) {
        WORKER_THREAD_STATE.with(|worker_thread_state| {
            let mut thread_state = worker_thread_state.borrow_mut();
            *thread_state = Some(current);
        });
    }

    pub fn push(&self, job: Job) {
        self.registry.thread_infos[self.index].worker.push(job);
    }

    pub fn pop(&self) -> Option<Job> {
        self.registry.thread_infos[self.index].worker.pop()
    }

    pub fn steal_until(&self, latch: Arc<Latch>) {
        while !latch.probe() {
            if let Some(mut job) = steal_work(self.registry.clone(), self.index) {
                job.execute();
            } else {
                thread::yield_now();
            }
        }
    }
}

fn main_loop(registry: Arc<Registry>, index: usize) {
    let worker_thread = Arc::new(WorkerThread {
        registry: registry.clone(),
        index,
    });
    WorkerThread::set_current(worker_thread.clone());
    registry.thread_infos[index].primed.set();
    let mut was_active = false;
    while !registry.terminated.load(Ordering::SeqCst) {
        if let Some(mut injected_job) = registry.wait_for_work(was_active) {
            registry.start_working();
            injected_job.execute();
            was_active = true;
        } else if let Some(mut stolen_job) = steal_work(registry.clone(), index) {
            registry.start_working();
            stolen_job.execute();
            was_active = true;
        } else {
            was_active = false;
        }
    }
}

fn steal_work(registry: Arc<Registry>, index: usize) -> Option<Job> {
    let num_threads = registry.num_threads();
    let start = rand::random::<usize>() % num_threads;
    (start..num_threads)
        .chain(0..start)
        .filter(|&i| i != index)
        .flat_map(|i| match registry.thread_infos[i].stealer.steal() {
            Stolen::Empty => None,
            Stolen::Abort => None,
            Stolen::Data(v) => Some(v),
        })
        .next()
}