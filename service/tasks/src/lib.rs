use std::hash::Hash;
use std::str::FromStr;
use std::time::Duration;

use prometheus::Registry;
use slog::Logger;

pub mod admin;
mod config;
mod error;
mod metrics;
mod request;
mod shared;
mod task_id;
mod worker;

pub use self::admin::TasksAdmin as Admin;
pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::request::TaskRequest;
pub use self::request::Tasks;
pub use self::task_id::TaskId;
pub use self::worker::Task;
pub use self::worker::TaskHandler;
pub use self::worker::WorkerSet;
pub use self::worker::WorkerSetPool;

#[cfg(any(debug_assertions, feature = "with_test_support"))]
pub use self::request::MockTasks;
#[cfg(any(debug_assertions, feature = "with_test_support"))]
pub use self::worker::mock as worker_mock;

/// Application defined queue definition.
///
/// Letting the application define a type for queues means that application can choose flexibility
/// (the TaskQueue is a String) or compile time checks (the TaskQueue is an enumeration).
///
/// Anything in between is also supported with variant attributes and complex structures.
pub trait TaskQueue:
    Clone + Eq + FromStr<Err = failure::Error> + Hash + Send + Sync + 'static
{
    /// The maximum number of retries for a task before skipping it.
    fn max_retry_count(&self) -> u8;

    /// The name of the queue tasks should be sent to/received from.
    fn name(&self) -> String;

    /// The delay before a failed task is retried.
    fn retry_delay(&self) -> Duration;
}

/// Attempts to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::metrics::register_metrics(logger, registry);
}
