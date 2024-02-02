//! Interface to initialise (both dependencies and client) for background task backends.
use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use crate::conf::Queue;
use crate::execute::TaskAck;
use crate::execute::TaskSource;
use crate::submit::Tasks;

/// Initialisation logic for Background Tasks and the client to access the service.
#[async_trait::async_trait]
pub trait TasksFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Initialise a [`TaskSource`] instance and a [`TaskAck`] instance for task processing.
    async fn consume<'a>(&self, args: TasksFactoryArgs<'a>) -> Result<(TaskSource, TaskAck)>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Initialise a [`Tasks`] instance to submit tasks for execution.
    async fn submit<'a>(&self, args: TasksFactoryArgs<'a>) -> Result<Tasks>;

    /// Synchronise (initialise or migrate) the Persistent store to handle [`Store`] operations.
    async fn sync<'a>(&self, args: TasksFactorySyncArgs<'a>) -> Result<()>;
}

/// Arguments passed to the [`TasksFactory`] client initialisation methods.
pub struct TasksFactoryArgs<'a> {
    /// The configuration block for the backend to initialise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}

/// Arguments passed to the [`TasksFactory`] client synchronisation method.
pub struct TasksFactorySyncArgs<'a> {
    /// The configuration block for the backend to synchronise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,

    /// All queues known to the process to be configured with the backend.
    pub queues: &'a Vec<&'static Queue>,
}
