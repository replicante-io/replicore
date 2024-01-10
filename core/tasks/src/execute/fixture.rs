use std::collections::HashSet;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

use replicore_context::Context;

use super::ReceivedTask;
use super::TaskAck;
use super::TaskAckBackend;
use super::TaskSource;
use super::TaskSourceBackend;
use crate::conf::Queue;

/// Fixed queue to submit unit test tasks to or receive them from.
pub static TEST_QUEUE: Lazy<Queue> = Lazy::new(|| Queue {
    queue: String::from("UNIT_TEST"),
    retry_count: 2,
    retry_timeout: Duration::from_millis(50),
});

/// Fixed queue to submit unit test tasks to or receive them from.
pub static TEST_QUEUE_ALTERNATE: Lazy<Queue> = Lazy::new(|| Queue {
    queue: String::from("UNIT_TEST_ALTERNATE"),
    retry_count: 0,
    retry_timeout: Duration::from_millis(10),
});

/// Fixed queue to return an error when tasks are received on it.
pub static TEST_FETCH_FAILURE: Lazy<Queue> = Lazy::new(|| Queue {
    queue: String::from("TEST_FETCH_FAILURE"),
    retry_count: 0,
    retry_timeout: Duration::from_millis(10),
});

/// Introspection tools to receive tasks during unit tests.
#[derive(Clone)]
pub struct ReceivedTaskFixture {
    done_count: Arc<AtomicU16>,
    send_task: Sender<ReceivedTask>,
}

impl ReceivedTaskFixture {
    /// Create a backend that will ack/nack tasks from this fixture.
    pub fn ack(&self) -> TaskAck {
        let ack = FixtureAckBackend {
            done_count: self.done_count.clone(),
        };
        TaskAck::from(ack)
    }

    /// Check the number of acks received by the backend.
    pub fn done_count(&self) -> u16 {
        self.done_count.load(Ordering::Relaxed)
    }

    /// Initialise a task queue backend fixture for unit tests.
    pub fn new() -> ReceivedTaskFixture {
        let (send_task, _) = broadcast::channel(50);
        ReceivedTaskFixture {
            done_count: Default::default(),
            send_task,
        }
    }

    /// Create a backend that will receive tasks from this fixture.
    pub fn source(&self) -> TaskSource {
        let source = FixtureSourceBackend {
            tasks: self.send_task.subscribe(),
            subscriptions: Default::default(),
        };
        TaskSource::from(source)
    }

    /// Send a task to the fixture backend.
    pub async fn submit(&self, task: ReceivedTask) -> Result<()> {
        self.send_task.send(task)?;
        Ok(())
    }
}

/// Tasks ack backend for unit tests.
pub struct FixtureAckBackend {
    done_count: Arc<AtomicU16>,
}

#[async_trait::async_trait]
impl TaskAckBackend for FixtureAckBackend {
    async fn done(&self, _: &Context, _: &ReceivedTask) -> Result<()> {
        self.done_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

/// Tasks source backend for unit tests.
pub struct FixtureSourceBackend {
    tasks: Receiver<ReceivedTask>,
    subscriptions: HashSet<String>,
}

#[async_trait::async_trait]
impl TaskSourceBackend for FixtureSourceBackend {
    async fn next(&mut self, _: &Context) -> Result<ReceivedTask> {
        loop {
            let next = self.tasks.recv().await?;
            if next.queue.queue == TEST_FETCH_FAILURE.queue {
                anyhow::bail!("fetch error task found in queue")
            }
            let subscribed = self.subscriptions.contains(&next.queue.queue);
            if !subscribed {
                continue;
            }
            return Ok(next);
        }
    }

    /// Configure the backend to subscribe to tasks submitted to a [`Queue`].
    async fn subscribe(&mut self, _: &Context, queue: &'static Queue) -> Result<()> {
        let queue = queue.queue.clone();
        self.subscriptions.insert(queue);
        Ok(())
    }
}
