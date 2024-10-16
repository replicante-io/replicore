//! Interface to submit tasks to a message queue platform.
use std::sync::Arc;

use anyhow::Result;
use opentelemetry_api::Context as OTelContext;
use serde::Serialize;
use serde_json::Value;

use replisdk::utils::metrics::CountFutureErrExt;

use replicore_context::Context;

use crate::conf::Queue;
use crate::conf::RunTaskAs;

/// Information about a task to submit for async execution.
#[derive(Clone, Debug)]
pub struct TaskSubmission {
    /// Payload submitted as part of this task.
    pub payload: Value,

    /// Queue the task is submitted to.
    pub queue: &'static Queue,

    /// Entity to use for authentication and authorisation when the task actually executes.
    pub run_as: Option<RunTaskAs>,

    /// OpenTelemetry context for trace data propagation.
    pub trace: Option<OTelContext>,
}

impl TaskSubmission {
    /// Collect information needed for task submission.
    pub fn new<P>(queue: &'static Queue, payload: &P) -> Result<TaskSubmission>
    where
        P: Serialize,
    {
        let task = TaskSubmission {
            payload: serde_json::to_value(payload)?,
            queue,
            run_as: None,
            trace: None,
        };
        Ok(task)
    }
}

/// Submit tasks to the backing task queue platform.
#[derive(Clone)]
pub struct Tasks(Arc<dyn TasksBackend>);

impl Tasks {
    /// Submit a task onto its queue.
    pub async fn submit<T>(&self, context: &Context, task: T) -> Result<()>
    where
        T: TryInto<TaskSubmission>,
        T::Error: Into<anyhow::Error>,
    {
        // Attach default contexts to propagate to tasks.
        let mut task = task.try_into().map_err(Into::into)?;
        if task.run_as.is_none() {
            task.run_as = context.auth.as_ref().map(RunTaskAs::from);
        }
        if task.trace.is_none() {
            task.trace = Some(OTelContext::current());
        }

        // Submit task while tracking telemetry.
        let queue_id = task.queue.queue.clone();
        let err_count = crate::telemetry::SUBMIT_ERR.with_label_values(&[&queue_id]);
        crate::telemetry::SUBMIT_COUNT
            .with_label_values(&[&queue_id])
            .inc();
        self.0.submit(context, task).count_on_err(err_count).await
    }

    /// Initialise a new tasks backend fixture for unit tests.
    #[cfg(feature = "test-fixture")]
    pub fn fixture() -> TasksFixture {
        TasksFixture::new()
    }
}

impl<T> From<T> for Tasks
where
    T: TasksBackend + 'static,
{
    fn from(value: T) -> Self {
        Tasks(Arc::new(value))
    }
}

/// Operations implemented by Message Queue Platforms supported by Replicante Core.
#[async_trait::async_trait]
pub trait TasksBackend: Send + Sync {
    /// Submit a task onto its queue.
    async fn submit(&self, context: &Context, task: TaskSubmission) -> Result<()>;
}

#[cfg(any(test, feature = "test-fixture"))]
pub use self::fixture::{TasksFixture, TasksFixtureBackend};

#[cfg(any(test, feature = "test-fixture"))]
mod fixture {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::sync::broadcast;
    use tokio::sync::broadcast::Receiver;
    use tokio::sync::broadcast::Sender;

    use replicore_context::Context;

    use super::TaskSubmission;
    use super::TasksBackend;

    /// Introspection tools for tasks submitted during unit tests.
    pub struct TasksFixture {
        tasks: Receiver<TaskSubmission>,
        send_task: Sender<TaskSubmission>,
    }

    impl Clone for TasksFixture {
        fn clone(&self) -> Self {
            let tasks = self.send_task.subscribe();
            Self {
                tasks,
                send_task: self.send_task.clone(),
            }
        }
    }

    impl TasksFixture {
        /// Create a backend that will send tasks to this fixture.
        pub fn backend(&self) -> TasksFixtureBackend {
            TasksFixtureBackend {
                send_task: self.send_task.clone(),
            }
        }

        /// Initialise a task queue backend fixture for unit tests.
        pub fn new() -> TasksFixture {
            let (send_task, tasks) = broadcast::channel(50);
            TasksFixture { tasks, send_task }
        }

        /// Fetch the next [`Task`] submitted to the fixture.
        pub async fn pop_task(&mut self) -> Result<TaskSubmission> {
            let task = self.tasks.recv().await?;
            Ok(task)
        }

        /// Fetch the next [`Task`] submitted to the fixture, with a timeout.
        pub async fn pop_task_timeout(&mut self, timeout: Duration) -> Result<TaskSubmission> {
            let task = tokio::time::timeout(timeout, self.pop_task()).await?;
            task
        }
    }

    /// Tasks backend for unit tests.
    pub struct TasksFixtureBackend {
        send_task: Sender<TaskSubmission>,
    }

    #[async_trait::async_trait]
    impl TasksBackend for TasksFixtureBackend {
        async fn submit(&self, _: &Context, task: TaskSubmission) -> Result<()> {
            self.send_task.send(task)?;
            Ok(())
        }
    }
}
