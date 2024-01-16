use std::{
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    mem,
    rc::Rc,
    task::{RawWaker, RawWakerVTable, Waker, Context}, pin::Pin,
};
use futures::{future::LocalBoxFuture, Future, FutureExt};
use crate::reactor::Reactor;

pub struct TaskQueue {
    queue: RefCell<VecDeque<Rc<Task>>>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        const DEFAULT_TASK_QUEUE_SIZE: usize = 4096;
        Self::new_with_capacity(DEFAULT_TASK_QUEUE_SIZE)
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queue: RefCell::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub(crate) fn push(&self, runnable: Rc<Task>) {
        self.queue.borrow_mut().push_back(runnable);
    }

    pub(crate) fn pop(&self) -> Option<Rc<Task>> {
        self.queue.borrow_mut().pop_front()
    }
}

pub struct Task {
    future: RefCell<LocalBoxFuture<'static, ()>>,
}

fn waker(wake: Rc<Task>) -> Waker {
    let ptr = Rc::into_raw(wake) as *const ();
    let vtable = &Helper::VTABLE;
    unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) }
}

impl Task {
    fn wake_(self: Rc<Self>) {
        Self::wake_by_ref_(&self)
    }

    fn wake_by_ref_(self: &Rc<Self>) {
        EX.with(|ex| ex.local_queue.push(self.clone()));
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static EX: Executor);

pub struct Executor {
    local_queue: TaskQueue,
    pub(crate) reactor: Rc<RefCell<Reactor>>,
    _marker: PhantomData<Rc<()>>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            local_queue: TaskQueue::default(),
            reactor: Rc::new(RefCell::new(Reactor::default())),
            _marker: PhantomData,
        }
    }

    pub fn spawn(fut: impl Future<Output = ()> + 'static) {
        let t = Rc::new(Task {
            future: RefCell::new(fut.boxed_local()),
        });
        EX.with(|ex| ex.local_queue.push(t));
    }

    pub fn block_on<F, T, O>(&self, f: F) -> O
    where
        F: Fn() -> T,
        T: Future<Output = O> + 'static,
    {
        let _waker = waker_fn::waker_fn(|| {});
        let cx = &mut Context::from_waker(&_waker);
        EX.set(self, || {
            let fut = f();
            pin_utils::pin_mut!(fut);
            loop {
                if let std::task::Poll::Ready(t) = fut.as_mut().poll(cx) {
                    break t;
                }
                while let Some(t) = self.local_queue.pop() {
                    let future = t.future.borrow_mut();
                    let w = waker(t.clone());
                    let mut context = Context::from_waker(&w);
                    let _ = Pin::new(future).as_mut().poll(&mut context);
                }
                if let std::task::Poll::Ready(t) = fut.as_mut().poll(cx) {
                    break t;
                }
                self.reactor.borrow_mut().wait();
            }
        })
    }
}

struct Helper;

impl Helper {
    const VTABLE: RawWakerVTable = RawWakerVTable::new (
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(data: *const ()) -> RawWaker {
        increase_refcount(data);
        let vtable = &Self::VTABLE;
        RawWaker::new(data, vtable)
    }

    unsafe fn wake(ptr: *const ()) {
        let rc = Rc::from_raw(ptr as *const Task);
        rc.wake_();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let rc = mem::ManuallyDrop::new(Rc::from_raw(ptr as *const Task));
        rc.wake_by_ref_();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Rc::from_raw(ptr as *const Task));
    }
}

#[allow(clippy::redundant_clone)]
unsafe fn increase_refcount(data: *const ()) {
    let rc = mem::ManuallyDrop::new(Rc::<Task>::from_raw(data as *const Task));
    let _rc_clone: mem::ManuallyDrop<_> = rc.clone();
}