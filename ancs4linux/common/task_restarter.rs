use std::time::Duration;
use tokio::time::sleep;

pub struct TaskRestarter<F, S, E>
where
    F: Fn() -> bool,
    S: Fn(),
    E: Fn(),
{
    max_attempts: usize,
    interval: Duration,
    fn: F,
    success_fn: S,
    failure_fn: E,
    attempts: usize,
}

impl<F, S, E> TaskRestarter<F, S, E>
where
    F: Fn() -> bool,
    S: Fn(),
    E: Fn(),
{
    pub fn new(max_attempts: usize, interval: Duration, fn: F, success_fn: S, failure_fn: E) -> Self {
        Self {
            max_attempts,
            interval,
            fn,
            success_fn,
            failure_fn,
            attempts: 0,
        }
    }

    pub async fn try_running_tick(&mut self) -> bool {
        if (self.fn)() {
            (self.success_fn)();
            return false;
        }

        self.attempts += 1;
        if self.attempts > self.max_attempts {
            (self.failure_fn)();
            return false;
        }

        // Retry.
        true
    }

    pub async fn try_running_bg(mut self) {
        while self.try_running_tick().await {
            sleep(self.interval).await;
        }
    }
}
