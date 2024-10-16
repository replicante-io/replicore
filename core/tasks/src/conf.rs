//! How to define queues and their configuration.
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

use replisdk::core::models::auth::AuthContext;
use replisdk::core::models::auth::Entity;
use replisdk::core::models::auth::ImpersonateEntity;

/// Definition of a task queue and its properties (such as retry logic).
///
/// Different queues are used to organise and group tasks to be executed and simplify
/// handling of tasks with different payloads/inputs.
#[derive(Debug)]
pub struct Queue {
    /// Identifier of the queue.
    pub queue: String,

    /// Number of times submitted tasks are retired in case of non-permanent failures.
    pub retry_count: u16,

    /// Amount of time a delivered task wait before redelivery attempts.
    pub retry_timeout: Duration,
}

/// Entity to use for authentication and authorisation when the task actually executes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct RunTaskAs {
    /// The entity (user or system) requesting and executing the task.
    pub entity: Entity,

    /// The entity to impersonate when processing the task.
    pub impersonate: Option<ImpersonateEntity>,
}

impl From<&AuthContext> for RunTaskAs {
    fn from(value: &AuthContext) -> Self {
        RunTaskAs {
            entity: value.entity.clone(),
            impersonate: value.impersonate.clone(),
        }
    }
}

/// Tasks executor backoff configuration in case of errors interacting with the Message Queue.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TasksExecutorBackoff {
    /// Maximum time, in seconds, to wait before retrying after errors from the Message Queue.
    #[serde(default = "TasksExecutorBackoff::default_max_delay")]
    pub max_delay: u64,

    /// Maximum number of retries before errors from the Message Queue cause process failure.
    #[serde(default = "TasksExecutorBackoff::default_max_retires")]
    pub max_retries: u16,

    /// Backoff multiplier every time a subsequent error is returned by the Message Queue.
    #[serde(default = "TasksExecutorBackoff::default_multiplier")]
    pub multiplier: u32,

    /// Initial delay, in milliseconds, to wait before the first retry.
    #[serde(default = "TasksExecutorBackoff::default_start_delay")]
    pub start_delay: u64,
}

impl Default for TasksExecutorBackoff {
    fn default() -> Self {
        TasksExecutorBackoff {
            max_delay: TasksExecutorBackoff::default_max_delay(),
            max_retries: TasksExecutorBackoff::default_max_retires(),
            multiplier: TasksExecutorBackoff::default_multiplier(),
            start_delay: TasksExecutorBackoff::default_start_delay(),
        }
    }
}

impl TasksExecutorBackoff {
    fn default_max_delay() -> u64 {
        30
    }

    fn default_max_retires() -> u16 {
        10
    }

    fn default_multiplier() -> u32 {
        2
    }

    fn default_start_delay() -> u64 {
        200
    }
}

/// Configuration for Background Tasks execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TasksExecutorConf {
    /// Tasks executor backoff configuration in case of errors interacting with the Message Queue.
    #[serde(default)]
    pub backoff: TasksExecutorBackoff,

    /// Maximum number of tasks to execute concurrently.
    #[serde(default = "TasksExecutorConf::default_concurrent_tasks")]
    pub concurrent_tasks: usize,

    /// Filter queues from which tasks should be processed.
    #[serde(default)]
    pub filters: TasksExecutorFilters,
}

impl Default for TasksExecutorConf {
    fn default() -> Self {
        TasksExecutorConf {
            backoff: Default::default(),
            concurrent_tasks: TasksExecutorConf::default_concurrent_tasks(),
            filters: Default::default(),
        }
    }
}

impl TasksExecutorConf {
    fn default_concurrent_tasks() -> usize {
        let parallel = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(8);
        parallel * 2
    }
}

/// Filter queues from which tasks should be processed.
///
/// These options allow runtime configuration of what processes should perform which work
/// and allow advanced topologies and provides a tool to scale across nodes.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TasksExecutorFilters {
    /// Ignore subscriptions to any task queue listed here.
    #[serde(default)]
    pub ignore: Vec<String>,

    /// If not empty, restrict subscriptions to only queues listed here.
    ///
    /// If the list is empty all queues can be subscribed to.
    #[serde(default)]
    pub process: Vec<String>,
}
