use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use super::Result;
use super::TaskError;
use super::TaskQueue;
use super::TaskId;
use super::metrics::TASK_ACK_ERRORS;
use super::metrics::TASK_ACK_TOTAL;

mod backend;
mod set;

#[cfg(debug_assertions)]
pub mod mock;

pub use self::backend::AckStrategy;
pub use self::set::TaskHandler;
pub use self::set::WorkerSet;
pub use self::set::WorkerSetPool;


/// Task information dispatched to a worker process.
///
/// Tasks are not `Send` or `Sync` due the `AckStrategy` not always being such.
/// The `WorkerSet` thread pool also works by using multiple threads to process one task each.
#[derive(Clone)]
pub struct Task<Q: TaskQueue> {
    pub(crate) ack_strategy: Arc<self::backend::AckStrategy<Q>>,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) id: TaskId,
    pub(crate) message: Vec<u8>,
    pub(crate) processed: bool,
    pub(crate) queue: Q,
    pub(crate) retry_count: u8,
}

impl<Q: TaskQueue> Task<Q> {
    /// Deserialise message into a structured object.
    pub fn deserialize<'de, D: Deserialize<'de>>(&'de self) -> Result<D> {
        let data = ::serde_json::from_slice(&self.message)?;
        Ok(data)
    }

    /// Lookup an header
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }

    /// Access the task ID.
    pub fn id(&self) -> &TaskId {
        &self.id
    }

    /// Access the message body
    pub fn message(&self) -> &[u8] {
        &self.message
    }
}

impl<Q: TaskQueue> Task<Q> {
    /// Mark the task as failed and needing retry
    ///
    /// If a task has failed too many times it will be skipped.
    pub fn fail(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        let queue = self.queue.name();
        if self.retry_count >= self.queue.max_retry_count() {
            TASK_ACK_TOTAL.with_label_values(&[&queue, "fail[skip]"]).inc();
            ack.skip(self).map_err(|error| {
                TASK_ACK_ERRORS.with_label_values(&[&queue, "fail[skip]"]).inc();
                error
            })
        } else {
            TASK_ACK_TOTAL.with_label_values(&[&queue, "fail"]).inc();
            ack.fail(self).map_err(|error| {
                TASK_ACK_ERRORS.with_label_values(&[&queue, "fail"]).inc();
                error
            })
        }
    }

    /// Permanently skip a task and move it aside for later inspection.
    pub fn skip(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        let queue = self.queue.name();
        TASK_ACK_TOTAL.with_label_values(&[&queue, "skip"]).inc();
        ack.skip(self).map_err(|error| {
            TASK_ACK_ERRORS.with_label_values(&[&queue, "skip"]).inc();
            error
        })
    }

    /// Mark the task as competed successfully
    pub fn success(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        let queue = self.queue.name();
        TASK_ACK_TOTAL.with_label_values(&[&queue, "success"]).inc();
        ack.success(self).map_err(|error| {
            TASK_ACK_ERRORS.with_label_values(&[&queue, "success"]).inc();
            error
        })
    }
}

impl<Q: TaskQueue> Drop for Task<Q> {
    fn drop(&mut self) {
        if !self.processed {
            panic!("task must be marked as process before they are dropped");
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::time::Duration;

    use super::TaskQueue;
    use super::mock::TaskAck;
    use super::mock::TaskTemplate;

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    enum TestQueues {
        Test1,
        Test2,
    }

    impl FromStr for TestQueues {
        type Err = ::failure::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "test1" => Ok(TestQueues::Test1),
                "test2" => Ok(TestQueues::Test2),
                s => Err(::failure::err_msg(format!("unknown queue '{}'", s))),
            }
        }
    }

    impl TaskQueue for TestQueues {
        fn max_retry_count(&self) -> u8 { 12 }
        fn name(&self) -> String {
            match self {
                TestQueues::Test1 => "test1".into(),
                TestQueues::Test2 => "test2".into(),
            }
        }
        fn retry_delay(&self) -> Duration { Duration::from_secs(5 * 60) }
    }

    #[test]
    #[should_panic(expected = "task must be marked as process before they are dropped")]
    fn unacked_task_cause_panic() {
        let template = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 0);
        let mock = template.mock();
        let _task = template.task();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::NotAcked);
    }

    #[test]
    fn task_fail() {
        let template = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 0);
        let mock = template.mock();
        let task = template.task();
        task.fail().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Fail);
    }

    #[test]
    fn task_skip() {
        let template = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 0);
        let mock = template.mock();
        let task = template.task();
        task.skip().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Skipped);
    }

    #[test]
    fn task_success() {
        let template = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 0);
        let mock = template.mock();
        let task = template.task();
        task.success().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Success);
    }

    #[test]
    fn too_many_retries_cause_skip() {
        let template = TaskTemplate::new(TestQueues::Test1, (), HashMap::new(), 100);
        let mock = template.mock();
        let task = template.task();
        task.fail().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Skipped);
    }
}
