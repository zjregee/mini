use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

use futures_core::ready;
use pin_project::pin_project;

use crate::backoff::{Backoff, BackoffBuilder};

pub trait Retryable<
    B: BackoffBuilder,
    T,
    E,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
> {
    fn retry(self, builder: &B) -> Retry<B::Backoff, T, E, Fut, FutureFn>;
}

impl<B, T, E, Fut, FutureFn> Retryable<B, T, E, Fut, FutureFn> for FutureFn
where
    B: BackoffBuilder,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
{
    fn retry(self, builder: &B) -> Retry<B::Backoff, T, E, Fut, FutureFn> {
        Retry::new(self, builder.build())
    }
}

#[pin_project]
pub struct Retry<
    B: Backoff,
    T,
    E,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
    RF = fn(&E) -> bool,
    NF = fn(&E, Duration),
> {
    backoff: B,
    retryable: RF,
    notify: NF,
    future_fn: FutureFn,
    #[pin]
    state: State<T, E, Fut>,
}

impl<B, T, E, Fut, FutureFn> Retry<B, T, E, Fut, FutureFn>
where
    B: Backoff,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
{
    fn new(future_fn: FutureFn, backoff: B) -> Self {
        Retry {
            backoff,
            retryable: |_: &E| true,
            notify: |_: &E, _: Duration| {},
            future_fn,
            state: State::Idle,
        }
    }
}

impl<B, T, E, Fut, FutureFn, RF, NF> Retry<B, T, E, Fut, FutureFn, RF, NF>
where
    B: Backoff,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
    RF: FnMut(&E) -> bool,
    NF: FnMut(&E, Duration),
{
    pub fn when <RN: FnMut(&E) -> bool> (
        self,
        retryable: RN,
    ) -> Retry<B, T, E, Fut, FutureFn, RN, NF> {
        Retry {
            backoff: self.backoff,
            retryable,
            notify: self.notify,
            future_fn: self.future_fn,
            state: self.state,
        }
    }
    pub fn notify <NN: FnMut(&E, Duration)> (
        self,
        notify: NN,
    ) -> Retry<B, T, E, Fut, FutureFn, RF, NN> {
        Retry {
            backoff: self.backoff,
            retryable: self.retryable,
            notify,
            future_fn: self.future_fn,
            state: self.state,
        }
    }
}

#[pin_project(project = StateProject)]
enum State<T, E, Fut: Future<Output = Result<T, E>>> {
    Idle,
    Polling(#[pin] Fut),
    Sleeping(#[pin] Pin<Box<tokio::time::Sleep>>),
}

impl<B, T, E, Fut, FutureFn, RF, NF> Future for Retry<B, T, E, Fut, FutureFn, RF, NF>
where
    B: Backoff,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
    RF: FnMut(&E) -> bool,
    NF: FnMut(&E, Duration),
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            let state = this.state.as_mut().project();
            match state {
                StateProject::Idle => {
                    let fut = (this.future_fn)();
                    this.state.set(State::Polling(fut));
                    continue;
                }
                StateProject::Polling(fut) => match ready!(fut.poll(cx)) {
                    Ok(v) => return Poll::Ready(Ok(v)),
                    Err(err) => {
                        if !(this.retryable)(&err) {
                            return Poll::Ready(Err(err));
                        }
                        match this.backoff.next() {
                            None => return Poll::Ready(Err(err)),
                            Some(dur) => {
                                (this.notify)(&err, dur);
                                this.state
                                    .set(State::Sleeping(Box::pin(tokio::time::sleep(dur))));
                                continue;
                            }
                        }
                    }
                },
                StateProject::Sleeping(sl) => {
                    ready!(sl.poll(cx));
                    this.state.set(State::Idle);
                    continue;
                }
            }
        }
    }
}