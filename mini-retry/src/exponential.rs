use std::time::Duration;

use crate::backoff::BackoffBuilder;

pub struct ExponentialBuilder {
    jitter: bool,
    factor: f32,
    min_delay: Duration,
    max_delay: Option<Duration>,
    max_times: Option<usize>,
}

impl Default for ExponentialBuilder {
    fn default() -> Self {
        Self {
            jitter: false,
            factor: 2.0,
            min_delay: Duration::from_secs(1),
            max_delay: Some(Duration::from_secs(60)),
            max_times: Some(3),
        }
    }
}

impl ExponentialBuilder {
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    pub fn with_factor(mut self, factor: f32) -> Self {
        self.factor = factor;
        self
    }

    pub fn with_min_delay(mut self, min_delay: Duration) -> Self {
        self.min_delay = min_delay;
        self
    }

    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = Some(max_delay);
        self
    }

    pub fn with_max_times(mut self, max_times: usize) -> Self {
        self.max_times = Some(max_times);
        self
    }
}


impl BackoffBuilder for ExponentialBuilder {
    type Backoff = ExponentialBackoff;

    fn build(&self) -> Self::Backoff {
        ExponentialBackoff {
            jitter: self.jitter,
            factor: self.factor,
            min_delay: self.min_delay,
            max_delay: self.max_delay,
            max_times: self.max_times,
            current_delay: None,
            attempts: 0,
        }
    }
}

pub struct ExponentialBackoff {
    jitter: bool,
    factor: f32,
    min_delay: Duration,
    max_delay: Option<Duration>,
    max_times: Option<usize>,
    current_delay: Option<Duration>,
    attempts: usize,
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.attempts >= self.max_times.unwrap_or(usize::MAX) {
            return None;
        }
        self.attempts += 1;
        match self.current_delay {
            None => {
                let mut cur = self.min_delay;
                self.current_delay = Some(cur);
                if self.jitter {
                    cur += self.min_delay.mul_f32(fastrand::f32());
                }
                Some(cur)
            }
            Some(mut cur) => {
                if let Some(max_delay) = self.max_delay {
                    if cur < max_delay {
                        cur = cur.mul_f32(self.factor);
                    }
                }
                self.current_delay = Some(cur);
                if self.jitter {
                    cur += self.min_delay.mul_f32(fastrand::f32());
                }
                Some(cur)
            }
        }
    }
}