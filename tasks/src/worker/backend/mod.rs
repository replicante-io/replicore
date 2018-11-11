use std::time::Duration;

use super::Result;
use super::Task;
use super::TaskQueue;

pub mod kafka;
#[cfg(debug_assertions)]
pub mod mock;


/// Backend specific task acknowledgement logic.
///
/// Once created, tasks are mostly independent of their backend.
/// Acks and retries are the exception.
pub trait AckStrategy<Q: TaskQueue>  : Send + Sync {
// TODO: remove sync and send and fix mock.
//pub trait AckStrategy<Q: TaskQueue> {
    /// Schedule the given task for retry because it failed.
    fn fail(&self, task: Task<Q>) -> Result<()>;

    /// Acknowledge the given task so we can move on to the next one.
    fn success(&self, task: Task<Q>) -> Result<()>;

    /// Copy the given task to a dedicated queue for later debugging.
    ///
    /// The task will not be retried any longer an may never succeed.
    fn trash(&self, task: Task<Q>) -> Result<()>;
}


/// Internal interface used to fetch tasks form the queue system backend.
///
/// This trait is used by the public interface but not exposed directly.
pub trait Backend<Q: TaskQueue> : Send + Sync {
    /// Attempt to fetch a new task, waiting at most `timeout` before giving up.
    fn poll(&self, timeout: Duration) -> Result<Option<Task<Q>>>;

    /// Subscribe to a queue for tasks to consume.
    fn subscribe(&mut self, queue: &Q) -> Result<()>;
}
