use std::sync::Arc;
use std::sync::Mutex;

use failure::ResultExt;
use serde_json;
use serde_json::Value;

use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::super::TaskQueue;
use super::super::TaskRequest;
use super::Backend;

pub type MockedRequests<Q> = Arc<Mutex<Vec<(TaskRequest<Q>, Value)>>>;

/// Mock implementation of a tasks queue backend.
pub struct Mock<Q: TaskQueue> {
    pub requests: MockedRequests<Q>,
}

impl<Q: TaskQueue> Backend<Q> for Mock<Q> {
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()> {
        let message: Value =
            serde_json::from_slice(message).with_context(|_| ErrorKind::PayloadDeserialize)?;
        self.requests
            .lock()
            .expect("failed to lock Mock#requests")
            .push((task, message));
        Ok(())
    }
}
