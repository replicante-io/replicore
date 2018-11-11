use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;

use super::AckStrategy;
use super::Backend;
use super::Result;
use super::Task;
use super::TaskQueue;


/// Strategy to operate on mock tasks.
pub struct MockAck {
    pub mock: Arc<Mutex<MockTask>>,
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


/// Mock implementation of a tasks queue backend.
pub struct MockBackend<Q: TaskQueue> {
    pub tasks: Arc<Mutex<VecDeque<TaskTemplate<Q>>>>,
}

impl<Q: TaskQueue> Backend<Q> for MockBackend<Q> {
    fn poll(&self, timeout: Duration) -> Result<Option<Task<Q>>> {
        // Simulate waiting for a task to arrive.
        sleep(timeout / 2);
        let task = self.tasks.lock().expect("mock tasks lock poisoned")
            .pop_front()
            .map(|t| t.task());
        Ok(task)
    }

    fn subscribe(&mut self, _queue: &Q) -> Result<()> {
        Ok(())
    }
}


/// Mock tools to simulate tasks.
pub struct MockTask {
    pub ack: TaskAck,
}


/// Track the acknowledgement state of a mocked task.
#[derive(Debug, Eq, PartialEq)]
pub enum TaskAck {
    Fail,
    NotAcked,
    Success,
    Trash,
}


/// TODO
pub struct TaskTemplate<Q: TaskQueue> {
    headers: HashMap<String, String>,
    message: Vec<u8>,
    mock: Arc<Mutex<MockTask>>,
    queue: Q,
    retry_count: u8,
}

impl<Q: TaskQueue> TaskTemplate<Q> {
    pub fn new<M>(
        queue: Q, message: M, headers: HashMap<String, String>, retry_count: u8
    ) -> TaskTemplate<Q>
        where M: Serialize
    {
        let mock = Arc::new(Mutex::new(MockTask {
            ack: TaskAck::NotAcked,
        }));
        TaskTemplate {
            headers,
            message: ::serde_json::to_vec(&message).expect("mock message to serialise"),
            mock,
            queue,
            retry_count,
        }
    }

    pub fn mock(&self) -> Arc<Mutex<MockTask>> {
        Arc::clone(&self.mock)
    }

    pub fn task(&self) -> Task<Q> {
        let ack_strategy = Arc::new(MockAck {
            mock: Arc::clone(&self.mock),
        });
        Task {
            ack_strategy, 
            headers: self.headers.clone(),
            message: self.message.clone(),
            processed: false,
            queue: self.queue.clone(),
            retry_count: self.retry_count,
        }
    }
}
