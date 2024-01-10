//! Track errors and successes to enable backing off retries.
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;

use replicore_context::Context;

use crate::conf::TasksExecutorBackoff;

/// Track failures and successes to incrementally delay retries.
///
/// The interface of the [`Backoff`] option is intended for use in looping operations:
///
/// - When a loop encounters an error call [`Backoff::retry`].
/// - When a loop completes call [`Backoff::success`] to clear memory of previous failures.
pub struct Backoff {
    delay: Duration,
    max_delay: Duration,
    max_retries: u16,
    multiplier: u32,
    seen: u16,
    start_delay: Duration,
}

impl Backoff {
    /// Initialise a new backoff engine.
    pub fn new(conf: &TasksExecutorBackoff) -> Backoff {
        let start_delay = Duration::from_millis(conf.start_delay);
        Backoff {
            delay: start_delay,
            max_delay: Duration::from_secs(conf.max_delay),
            max_retries: conf.max_retries,
            multiplier: conf.multiplier,
            seen: 0,
            start_delay,
        }
    }

    /// The loop has encountered an error and needs to delay the next cycle appropriately.
    ///
    /// When the loop fails too many time the original error is reported back to fail properly.
    /// Otherwise the function sleeps for an incrementally longer period, up to a configured max.
    pub async fn retry(&mut self, context: &Context, error: Error) -> Result<()> {
        self.seen += 1;
        if self.seen > self.max_retries {
            let context = crate::error::RetriesExceeded::new(self.max_retries);
            let error = error.context(context);
            return Err(error);
        }

        slog::warn!(
            context.logger, "Encounter error while processing async task, will retry";
            replisdk::utils::error::slog::ErrorAttributes::from(&error),
        );
        tokio::time::sleep(self.delay).await;
        self.delay = std::cmp::min(self.delay * self.multiplier, self.max_delay);
        Ok(())
    }

    /// Reset the state of tracked failures.
    pub fn success(&mut self) {
        self.delay = self.start_delay;
        self.seen = 0;
    }
}
