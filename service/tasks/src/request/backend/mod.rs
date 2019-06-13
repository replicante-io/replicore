use super::Result;
use super::TaskQueue;
use super::TaskRequest;

pub mod kafka;
#[cfg(debug_assertions)]
pub mod mock;

/// Internal interface used to request tasks form the queue system backend.
///
/// This trait is used by the public interface but not exposed directly.
pub trait Backend<Q: TaskQueue>: Send + Sync {
    /// Sends a task with a payload to the queue system.
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()>;
}
