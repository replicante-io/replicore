use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;

use replicore_context::Context;

use super::ReceivedTask;
use super::ReceivedTaskFixture;
use super::TaskCallback;
use super::TasksExecutor;
use super::TEST_FETCH_FAILURE;
use super::TEST_QUEUE;
use super::TEST_QUEUE_ALTERNATE;
use crate::conf::TasksExecutorConf;

/// Task handler that can return errors.
pub enum AckTask {
    /// Fail handling with a permanent error.
    FailPermanent,

    /// Fail handling with a generic error.
    FailRetry,

    /// Succeed handling without error.
    Succeed,
}

#[async_trait::async_trait]
impl TaskCallback for AckTask {
    async fn execute(&self, _: &Context, _: &ReceivedTask) -> Result<()> {
        match self {
            AckTask::FailPermanent => {
                let error = anyhow::anyhow!("test error");
                let error = error.context(crate::error::AbandonTask);
                anyhow::bail!(error)
            }
            AckTask::FailRetry => anyhow::bail!(anyhow::anyhow!("test error")),
            AckTask::Succeed => Ok(()),
        }
    }
}

/// Simple task handler that counts its invocations.
#[derive(Clone, Default)]
struct Counter(Arc<AtomicU16>);

impl Counter {
    /// Return a reference to the inner atomic counter.
    pub fn atomic(&self) -> Arc<AtomicU16> {
        self.0.clone()
    }
}

#[async_trait::async_trait]
impl TaskCallback for Counter {
    async fn execute(&self, _: &Context, _: &ReceivedTask) -> Result<()> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

/// Set of fixtures to use in tests.
pub struct Fixtures {
    context: Context,
    executor: TasksExecutor,
    exit: tokio::time::Sleep,
    tasks: ReceivedTaskFixture,
}

impl Fixtures {
    /// Initialise a set of test fixtures.
    pub fn new() -> Fixtures {
        let mut conf = TasksExecutorConf::default();
        conf.backoff.max_retries = 0;
        Fixtures::with_conf(conf)
    }

    /// Initialise a set of test fixtures with a set configuration.
    pub fn with_conf(conf: TasksExecutorConf) -> Fixtures {
        let tasks = ReceivedTaskFixture::new();
        let executor = TasksExecutor::new(tasks.source(), tasks.ack(), conf);
        Fixtures {
            context: Context::fixture(),
            executor,
            exit: tokio::time::sleep(std::time::Duration::from_millis(100)),
            tasks,
        }
    }
}

/// Simple task handler that counts its invocations.
struct Panic;

#[async_trait::async_trait]
impl TaskCallback for Panic {
    async fn execute(&self, _: &Context, _: &ReceivedTask) -> Result<()> {
        panic!("test panic propagation")
    }
}

/// Task to sleep before completing.
struct Sleep;

#[async_trait::async_trait]
impl TaskCallback for Sleep {
    async fn execute(&self, _: &Context, task: &ReceivedTask) -> Result<()> {
        let duration = task.payload.as_u64().unwrap();
        let duration = std::time::Duration::from_millis(duration);
        tokio::time::sleep(duration).await;
        Ok(())
    }
}

#[tokio::test]
async fn acknowledge_fail_permanent() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, AckTask::FailPermanent)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    assert_eq!(1, fixtures.tasks.done_count());
}

#[tokio::test]
async fn acknowledge_fail_retry() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, AckTask::FailRetry)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    assert_eq!(0, fixtures.tasks.done_count());
}

#[tokio::test]
async fn acknowledge_success() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, AckTask::Succeed)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    assert_eq!(1, fixtures.tasks.done_count());
}

#[tokio::test]
async fn cancel_task_on_exit() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, Sleep)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "long".into(),
        payload: serde_json::json!(200u64),
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    let task = ReceivedTask {
        id: "short".into(),
        payload: serde_json::json!(50u64),
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();

    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(1, fixtures.tasks.done_count());
}

#[tokio::test]
async fn cancel_task_on_error() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, Sleep)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "error".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_FETCH_FAILURE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    let task = ReceivedTask {
        id: "short".into(),
        payload: serde_json::json!(50u64),
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();

    let result = executor.execute(&fixtures.context, fixtures.exit).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(0, fixtures.tasks.done_count());
    assert!(result.is_err());
}

#[tokio::test]
async fn dispatch_task() {
    let counter = Counter::default();
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;

    // Subscribe to task queue.
    let counter_atomic = counter.atomic();
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, counter)
        .await
        .unwrap();

    // Submit tasks for test.
    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    let task_alternate = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE_ALTERNATE,
    };
    fixtures.tasks.submit(task.clone()).await.unwrap();
    fixtures.tasks.submit(task).await.unwrap();
    fixtures.tasks.submit(task_alternate).await.unwrap();

    // Execute tasks handler and assert results.
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    let count = counter_atomic.load(Ordering::Relaxed);
    assert_eq!(count, 2);
}

#[should_panic(expected = "test panic propagation")]
#[tokio::test]
async fn panic_propagates() {
    let fixtures = Fixtures::new();
    let mut executor = fixtures.executor;
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, Panic)
        .await
        .unwrap();

    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
}

#[tokio::test]
async fn subscriptions_are_filtered() {
    let counter = Counter::default();
    let mut conf = TasksExecutorConf::default();
    conf.backoff.max_retries = 0;
    conf.filters.process = vec![TEST_QUEUE_ALTERNATE.queue.clone()];
    let fixtures = Fixtures::with_conf(conf);
    let mut executor = fixtures.executor;

    // Subscribe to task queue.
    let counter_atomic = counter.atomic();
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE, counter.clone())
        .await
        .unwrap();
    executor
        .subscribe(&fixtures.context, &TEST_QUEUE_ALTERNATE, counter)
        .await
        .unwrap();

    // Submit tasks for test.
    let task = ReceivedTask {
        id: "test".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE,
    };
    let task_alternate = ReceivedTask {
        id: "filtered".into(),
        payload: serde_json::Value::Null,
        queue: &TEST_QUEUE_ALTERNATE,
    };
    fixtures.tasks.submit(task).await.unwrap();
    fixtures.tasks.submit(task_alternate).await.unwrap();

    // Execute tasks handler and assert results.
    executor
        .execute(&fixtures.context, fixtures.exit)
        .await
        .unwrap();
    let count = counter_atomic.load(Ordering::Relaxed);
    assert_eq!(count, 1);
}
