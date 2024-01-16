use std::{
    cell::RefCell,
    os::unix::prelude::{AsRawFd, RawFd},
    rc::Rc,
    task::{Context, Waker},
};
use polling::{Event, Poller};

pub(crate) fn get_reactor() -> Rc<RefCell<Reactor>> {
    crate::executor::EX.with(|ex| ex.reactor.clone())
}

pub struct Reactor {
    poller: Poller,
    waker_mapping: rustc_hash::FxHashMap<u64, Waker>,
    buffer: Vec<Event>,
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Reactor {
    pub fn new() -> Self {
        Self {
            poller: Poller::new().unwrap(),
            waker_mapping: Default::default(),
            buffer: Vec::with_capacity(2048),
        }
    }

    pub fn add(&mut self, fd: RawFd) {
        let flags = nix::fcntl::OFlag::from_bits(nix::fcntl::fcntl(fd, nix::fcntl::F_GETFL).unwrap()).unwrap();
        let flags_nonblocking = flags | nix::fcntl::OFlag::O_NONBLOCK;
        nix::fcntl::fcntl(fd, nix::fcntl::F_SETFL(flags_nonblocking)).unwrap();
        self.poller.add(fd, polling::Event::none(fd as usize)).unwrap();
    }

    pub fn modify_readable(&mut self, fd: RawFd, cx: &mut Context) {
        self.push_completion(fd as u64 * 2, cx);
        let event = polling::Event::readable(fd as usize);
        self.poller.modify(fd, event);
    }

    pub fn modify_writable(&mut self, fd: RawFd, cx: &mut Context) {
        self.push_completion(fd as u64 * 2 + 1, cx);
        let event = polling::Event::writable(fd as usize);
        self.poller.modify(fd, event);
    }

    pub fn wait(&mut self) {
        self.poller.wait(&mut self.buffer, None);
        for i in 0..self.buffer.len() {
            let event = self.buffer.swap_remove(0);
            if event.readable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2)) {
                    waker.wake();
                }
            }
            if event.writable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2 + 1)) {
                    waker.wake();
                }
            }
        }
    }

    pub fn delete(&mut self, fd: RawFd) {
        self.waker_mapping.remove(&(fd as u64 * 2));
        self.waker_mapping.remove(&(fd as u64 * 2 + 1));
    }

    fn push_completion(&mut self, token: u64, cx: &mut Context) {
        self.waker_mapping.insert(token, cx.waker().clone());
    }
}