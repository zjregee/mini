use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Latch {
    b: AtomicBool,
}

impl Latch {
    pub fn new() -> Latch {
        Latch {
            b: AtomicBool::new(false)
        }
    }

    pub fn set(&self) {
        self.b.store(true, Ordering::SeqCst);
    }

    pub fn wait(&self) {
        while !self.probe() {
            thread::yield_now();
        }
    }

    pub fn probe(&self) -> bool {
        self.b.load(Ordering::SeqCst)
    }
}