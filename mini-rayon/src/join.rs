use std::sync::Arc;
use crate::latch::Latch;
use crate::job::{Code, Job};
use crate::thread_pool::{WorkerThread, Registry, get_registry};

pub fn join<A, RA, B, RB>(oper_a: A, oper_b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send + 'static,
    B: FnOnce() -> RB + Send + 'static,
    RA: Send + 'static,
    RB: Send + 'static,
{
    let worker_thread = WorkerThread::current();
    if worker_thread.is_none() {
        return join_inject(oper_a, oper_b);
    }
    let worker_thread = worker_thread.unwrap();
    let mut result_b = None;
    let code_b = Code::new(oper_b, &mut result_b as *mut Option<RB>);
    let latch_b = Arc::new(Latch::new());
    let job_b = Job::new(Box::new(code_b), latch_b.clone());
    worker_thread.push(job_b);
    let result_a = oper_a();
    if let Some(mut job_b) = worker_thread.pop() {
        job_b.execute();
    } else {
        worker_thread.steal_until(latch_b);
    }
    (result_a, result_b.unwrap())
}

fn join_inject<A, RA, B, RB>(oper_a: A, oper_b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send + 'static,
    B: FnOnce() -> RB + Send + 'static,
    RA: Send + 'static,
    RB: Send + 'static,
{
    let mut result_a = None;
    let code_a = Code::new(oper_a, &mut result_a as *mut Option<RA>);
    let latch_a = Arc::new(Latch::new());
    let job_a = Job::new(Box::new(code_a), latch_a.clone());
    let mut result_b = None;
    let code_b = Code::new(oper_b, &mut result_b as *mut Option<RB>);
    let latch_b = Arc::new(Latch::new());
    let job_b = Job::new(Box::new(code_b), latch_b.clone());
    get_registry().inject(vec![job_a, job_b]);
    latch_a.wait();
    latch_b.wait();
    (result_a.unwrap(), result_b.unwrap())
}

pub struct ThreadPool {
    registry: Arc<Registry>,
}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        ThreadPool {
            registry: Registry::new(),
        }
    }

    pub fn install<OP, R>(&self, op: OP) -> R
    where
        OP: Fn() -> R + Send + 'static,
        R: Send + 'static,
    {
        let mut result = None;
        let code = Code::new(op, &mut result as *mut Option<R>);
        let latch = Arc::new(Latch::new());
        let job = Job::new(Box::new(code), latch.clone());
        self.registry.inject(vec![job]);
        latch.wait();
        result.unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.registry.terminate();
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_join() {

    }

    #[test]
    fn test_thread_pool() {

    }
}