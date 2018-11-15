use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use super::Result;
use super::TaskError;
use super::TaskQueue;

mod backend;
mod set;

#[cfg(debug_assertions)]
pub mod mock;

pub use self::set::TaskHandler;
pub use self::set::WorkerSet;
pub use self::set::WorkerSetPool;


// TODO: replace with configurable option.
const MAX_RETRY_COUNT: u8 = 12;


/// Task information dispatched to a worker process.
///
/// Tasks are not `Send` or `Sync` due the `AckStrategy` not always being such.
/// The `WorkerSet` thread pool also works by using multiple threads to process one task each.
#[derive(Clone)]
pub struct Task<Q: TaskQueue> {
    ack_strategy: Arc<self::backend::AckStrategy<Q>>,
    headers: HashMap<String, String>,
    message: Vec<u8>,
    processed: bool,
    queue: Q,
    retry_count: u8,
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
        if self.retry_count >= MAX_RETRY_COUNT {
            ack.skip(self)
        } else {
            ack.fail(self)
        }
    }

    /// Permanently skip a task and move it aside for later inspection.
    pub fn skip(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        ack.skip(self)
    }

    /// Mark the task as competed successfully
    pub fn success(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        ack.success(self)
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
        fn name(&self) -> String {
            match self {
                TestQueues::Test1 => "test1".into(),
                TestQueues::Test2 => "test2".into(),
            }
        }
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
