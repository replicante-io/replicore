//! Logic and interface to receive and execute submitted tasks.
use std::sync::Arc;

use anyhow::Result;
use opentelemetry_api::Context as OTelContext;
use serde::de::DeserializeOwned;
use serde_json::Value;

use replisdk::utils::metrics::CountFutureErrExt;

use replicore_context::Context;

mod backoff;
mod executor;

pub use self::executor::TasksExecutor;
pub use self::executor::TasksExecutorBuilder;

#[cfg(any(test, feature = "test-fixture"))]
mod fixture;
#[cfg(any(test, feature = "test-fixture"))]
pub use self::fixture::{
    FixtureSourceBackend, ReceivedTaskFixture, TEST_FETCH_FAILURE, TEST_QUEUE, TEST_QUEUE_ALTERNATE,
};

#[cfg(test)]
mod tests;

use crate::conf::Queue;
use crate::conf::RunTaskAs;

/// Information about a received task that needs executing.
#[derive(Clone, Debug)]
pub struct ReceivedTask {
    /// ID of the received task (as determined by the queuing backend).
    pub id: String,

    /// Payload received for this task.
    pub payload: Value,

    /// Queue the task was received from.
    pub queue: &'static Queue,

    /// Entity to use for authentication and authorisation when the task actually executes.
    pub run_as: Option<RunTaskAs>,

    /// OpenTelemetry context for trace data propagation.
    pub trace: Option<OTelContext>,
}

impl ReceivedTask {
    /// Decode task payload into a structured item.
    pub fn decode<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let payload = serde_json::from_value(self.payload.clone())?;
        Ok(payload)
    }
}

/// Notify the backing queue platform of updates to tasks.
#[derive(Clone)]
pub struct TaskAck(Arc<dyn TaskAckBackend>);

impl TaskAck {
    /// Mark the task as processed (either successfully or not) so it can be removed.
    pub async fn done(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        self.0.done(context, task).await
    }
}

impl<T> From<T> for TaskAck
where
    T: TaskAckBackend + 'static,
{
    fn from(value: T) -> Self {
        TaskAck(Arc::new(value))
    }
}

/// Operations to notify the Message Queue Platforms about task processing update.
#[async_trait::async_trait]
pub trait TaskAckBackend: Send + Sync {
    /// Mark the task as processed (either successfully or not) so it can be removed.
    async fn done(&self, context: &Context, task: &ReceivedTask) -> Result<()>;
}

/// Async callback invoked to execute received tasks.
#[async_trait::async_trait]
pub trait TaskCallback: Send + Sync {
    /// Execute task logic on the received task.
    async fn execute(&self, context: &Context, task: &ReceivedTask) -> Result<()>;
}

/// Receive tasks from the backing queue platform so they can be executed.
pub struct TaskSource(Box<dyn TaskSourceBackend>);

impl TaskSource {
    /// Fetch the next task available for processing.
    pub async fn next(&mut self, context: &Context) -> Result<ReceivedTask> {
        let err_count = crate::telemetry::RECEIVE_ERR.clone();
        crate::telemetry::RECEIVE_COUNT.inc();
        self.0.next(context).count_on_err(err_count).await
    }

    /// Configure the backend to subscribe to tasks submitted to a [`Queue`].
    pub async fn subscribe(&mut self, context: &Context, queue: &'static Queue) -> Result<()> {
        self.0.subscribe(context, queue).await
    }
}

impl<T> From<T> for TaskSource
where
    T: TaskSourceBackend + 'static,
{
    fn from(value: T) -> Self {
        TaskSource(Box::new(value))
    }
}

/// Operations to receive tasks from Message Queue Platforms supported by Replicante Core.
#[async_trait::async_trait]
pub trait TaskSourceBackend: Send + Sync {
    /// Fetch the next task available for processing.
    async fn next(&mut self, context: &Context) -> Result<ReceivedTask>;

    /// Configure the backend to subscribe to tasks submitted to a [`Queue`].
    async fn subscribe(&mut self, context: &Context, queue: &'static Queue) -> Result<()>;
}
