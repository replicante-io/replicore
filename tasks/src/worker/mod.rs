use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use super::Result;
use super::TaskError;
use super::TaskQueue;

mod backend;
mod set;

pub use self::set::WorkerSet;
pub use self::set::WorkerSetPool;

#[cfg(debug_assertions)]
pub use self::backend::mock;
#[cfg(debug_assertions)]
pub use self::set::MockWorkerSet;


// TODO: replace with configurable option.
const MAX_RETRY_COUNT: u8 = 10;


/// Mock tools to simulate tasks.
#[cfg(debug_assertions)]
pub struct MockTask {
    pub ack: mock::TaskAck,
}

#[cfg(debug_assertions)]
impl MockTask {
    pub fn mock<M, Q>(
        queue: Q, message: M, headers: HashMap<String, String>, retry_count: u8
    ) -> Result<(Task<Q>, Arc<::std::sync::Mutex<MockTask>>)>
        where M: ::serde::Serialize,
              Q: TaskQueue,
    {
        let mock = Arc::new(::std::sync::Mutex::new(MockTask {
            ack: mock::TaskAck::NotAcked,
        }));
        let ack_strategy = Arc::new(mock::MockAck {
            mock: Arc::clone(&mock)
        });
        let message = ::serde_json::to_vec(&message)?;
        let task = Task {
            ack_strategy,
            headers,
            message,
            processed: false,
            queue,
            retry_count: retry_count,
        };
        Ok((task, mock))
    }
}


/// Task information dispatched to a worker process.
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
    pub fn fail(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        if self.retry_count >= MAX_RETRY_COUNT {
            ack.trash(self)
        } else {
            ack.fail(self)
        }
    }

    /// Mark the task as competed successfully
    pub fn success(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        ack.success(self)
    }

    /// Move a failed task to the (backend dependent) trash
    pub fn trash(mut self) -> Result<()> {
        self.processed = true;
        let ack = Arc::clone(&self.ack_strategy);
        ack.trash(self)
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

    use super::MockTask;
    use super::TaskQueue;
    use super::mock::TaskAck;

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
        let (_task, mock) = MockTask::mock(TestQueues::Test1, (), HashMap::new(), 0).unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::NotAcked);
    }

    #[test]
    fn task_fail() {
        let (task, mock) = MockTask::mock(TestQueues::Test1, (), HashMap::new(), 0).unwrap();
        task.fail().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Fail);
    }

    #[test]
    fn task_success() {
        let (task, mock) = MockTask::mock(TestQueues::Test1, (), HashMap::new(), 0).unwrap();
        task.success().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Success);
    }

    #[test]
    fn task_trash() {
        let (task, mock) = MockTask::mock(TestQueues::Test1, (), HashMap::new(), 0).unwrap();
        task.trash().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Trash);
    }

    #[test]
    fn too_many_retries_cause_trash() {
        let (task, mock) = MockTask::mock(TestQueues::Test1, (), HashMap::new(), 100).unwrap();
        task.fail().unwrap();
        assert_eq!(mock.lock().unwrap().ack, TaskAck::Trash);
    }
}
