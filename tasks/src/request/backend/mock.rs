use std::sync::Arc;
use std::sync::Mutex;

use serde_json;
use serde_json::Value;

use super::Backend;
use super::Result;
use super::TaskQueue;
use super::TaskRequest;


/// Mock implementation of a tasks queue backend.
pub struct Mock<Q: TaskQueue> {
    pub requests: Arc<Mutex<Vec<(TaskRequest<Q>, Value)>>>,
}

impl<Q: TaskQueue> Backend<Q> for Mock<Q> {
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()> {
        let message: Value = serde_json::from_slice(message)?;
        self.requests.lock().expect("failed to lock Mock#requests").push((task, message));
        Ok(())
    }
}
