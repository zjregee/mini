use std::sync::Arc;
use crate::latch::Latch;

pub trait Executable {
    fn execute(&mut self);
}

pub struct Code<F, R> {
    func: Option<F>,
    dest: *mut Option<R>,
}

impl<F, R> Code<F, R>
where
    F: FnOnce() -> R,
{
    pub fn new(func: F, dest: *mut Option<R>) -> Code<F, R> {
        Code {
            func: Some(func),
            dest,
        }
    }
}

impl<F, R> Executable for Code<F, R>
where
    F: FnOnce() -> R,
{
    fn execute(&mut self) {
        if let Some(func) = self.func.take() {
            unsafe {
                *self.dest = Some(func());
            }
        }
    }
}

pub struct Job {
    code: Box<dyn Executable>,
    latch: Arc<Latch>,
}

impl Job {
    pub fn new(code: Box<dyn Executable>, latch: Arc<Latch>) -> Job {
        Job {
            code,
            latch,
        }
    }

    pub fn execute(&mut self) {
        self.code.execute();
        self.latch.set();
    }
}

unsafe impl Send for Job { }
unsafe impl Sync for Job { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let closure = || {
            42
        };
        let mut result = None;
        let mut code = Code::new(closure, &mut result);
        code.execute();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_job() {
        let closure = || {
            42
        };
        let mut result = None;
        let code = Code::new(closure, &mut result);
        let latch = Arc::new(Latch::new());
        let mut job = Job::new(Box::new(code), latch.clone());
        assert_eq!(latch.probe(), false);
        job.execute();
        assert_eq!(latch.probe(), true);
        assert_eq!(result.unwrap(), 42);
    }
}