use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use super::AckStrategy;
use super::Backend;
use super::Result;
use super::Task;
use super::TaskQueue;


/// Mock implementation of a tasks queue backend.
pub struct Mock<Q: TaskQueue> {
    pub tasks: Arc<Mutex<VecDeque<Task<Q>>>>,
}

impl<Q: TaskQueue> Backend<Q> for Mock<Q> {
    fn poll(&self, timeout: Duration) -> Result<Option<Task<Q>>> {
        // Simulate waiting for a task to arrive.
        sleep(timeout / 2);
        let task = self.tasks.lock().expect("mock tasks lock poisoned").pop_front();
        Ok(task)
    }

    fn subscribe(&mut self, _queue: &Q) -> Result<()> {
        Ok(())
    }
}


/// Strategy to operate on mock tasks.
pub struct MockAck {
    pub mock: Arc<Mutex<super::super::MockTask>>,
}

impl<Q: TaskQueue> AckStrategy<Q> for MockAck {
    fn fail(&self, _task: Task<Q>) -> Result<()> {
        self.mock.lock().unwrap().ack = TaskAck::Fail;
        Ok(())
    }

    fn success(&self, _task: Task<Q>) -> Result<()> {
        self.mock.lock().unwrap().ack = TaskAck::Success;
        Ok(())
    }

    fn trash(&self, _task: Task<Q>) -> Result<()> {
        self.mock.lock().unwrap().ack = TaskAck::Trash;
        Ok(())
    }
}


/// Track the acknowledgement state of a mocked task.
#[derive(Debug, Eq, PartialEq)]
pub enum TaskAck {
    Fail,
    NotAcked,
    Success,
    Trash,
}
