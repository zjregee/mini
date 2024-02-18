use std::thread;
use std::time::Duration;

use crate::backoff::BackoffBuilder;
use crate::Backoff;

pub trait BlockingRetryable<B: BackoffBuilder, T, E, F: FnMut() -> Result<T, E>> {
    fn retry(self, builder: &B) -> BlockingRetry<B::Backoff, T, E, F>;
}

impl<B, T, E, F> BlockingRetryable<B, T, E, F> for F
where
    B: BackoffBuilder,
    F: FnMut() -> Result<T, E>,
{
    fn retry(self, builder: &B) -> BlockingRetry<B::Backoff, T, E, F> {
        BlockingRetry::new(self, builder.build())
    }
}

pub struct BlockingRetry<
    B: Backoff,
    T,
    E,
    F: FnMut() -> Result<T, E>,
    RF = fn(&E) -> bool,
    NF = fn(&E, Duration),
> {
    backoff: B,
    retryable: RF,
    notify: NF,
    f: F,
}

impl<B, T, E, F> BlockingRetry<B, T, E, F>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
{
    fn new(f: F, backoff: B) -> Self {
        BlockingRetry {
            backoff,
            retryable: |_: &E| true,
            notify: |_: &E, _: Duration| {},
            f,
        }
    }
}

impl<B, T, E, F, RF, NF> BlockingRetry<B, T, E, F, RF, NF>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
    RF: FnMut(&E) -> bool,
    NF: FnMut(&E, Duration),
{
    pub fn when<RN: FnMut(&E) -> bool>(self, retryable: RN) -> BlockingRetry<B, T, E, F, RN, NF> {
        BlockingRetry {
            backoff: self.backoff,
            retryable,
            notify: self.notify,
            f: self.f,
        }
    }

    pub fn notify<NN: FnMut(&E, Duration)>(self, notify: NN) -> BlockingRetry<B, T, E, F, RF, NN> {
        BlockingRetry {
            backoff: self.backoff,
            retryable: self.retryable,
            notify,
            f: self.f,
        }
    }

    pub fn call(mut self) -> Result<T, E> {
        loop {
            let result = (self.f)();
            match result {
                Ok(v) => return Ok(v),
                Err(err) => {
                    if !(self.retryable)(&err) {
                        return Err(err);
                    }
                    match self.backoff.next() {
                        None => return Err(err),
                        Some(dur) => {
                            (self.notify)(&err, dur);
                            thread::sleep(dur);
                        }
                    }
                }
            }
        }
    }
}