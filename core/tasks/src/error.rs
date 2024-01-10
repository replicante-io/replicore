//! Errors reported by the async task framework.

/// background task encountered a permanent error and will not be retired.
#[derive(Debug, thiserror::Error)]
#[error("background task encountered a permanent error and will not be retired")]
pub struct AbandonTask;

/// Task executor already subscribed to queue.
#[derive(Debug, thiserror::Error)]
#[error("task executor already subscribed to queue '{0}'")]
pub struct AlreadySubscribed(&'static str);

impl AlreadySubscribed {
    /// Report the given task queue is already subscribed to with another handler.
    pub fn new(queue: &'static str) -> AlreadySubscribed {
        AlreadySubscribed(queue)
    }
}

/// Exceeded maximum number of retries.
#[derive(Debug, thiserror::Error)]
#[error("exceeded maximum of {0} retries")]
pub struct RetriesExceeded(u16);

impl RetriesExceeded {
    /// Report the given number of retries was exceeded.
    pub fn new(max: u16) -> RetriesExceeded {
        RetriesExceeded(max)
    }
}
