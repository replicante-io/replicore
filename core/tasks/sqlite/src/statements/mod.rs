//! SQL statements to implement the [`TasksBackend`] with SQLite.
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_tasks::conf::Queue;
use replicore_tasks::execute::ReceivedTask;
use replicore_tasks::execute::TaskAckBackend;
use replicore_tasks::execute::TaskSourceBackend;
use replicore_tasks::submit::TaskSubmission;
use replicore_tasks::submit::TasksBackend;

mod execute;
mod submit;

/// Implementation of the [`TasksBackend`] interface using SQLite.
#[derive(Clone)]
pub struct SQLiteTasks {
    /// Connection to the SQLite DB persisting data.
    connection: Connection,

    /// Delay between DB queries for pending/retry tasks to become available for execution.
    pub poll_delay: Duration,

    /// List of queue IDs a client is subscribed to (for TaskSourceBackend).
    subscriptions: HashMap<&'static String, &'static Queue>,
}

impl SQLiteTasks {
    /// Initialise a new SQLite backed [`TasksBackend`].
    pub fn new(connection: Connection, conf: &crate::Conf) -> Self {
        SQLiteTasks {
            connection,
            poll_delay: Duration::from_secs(conf.poll_delay_s),
            subscriptions: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl TaskAckBackend for SQLiteTasks {
    async fn done(&self, context: &Context, task: &ReceivedTask) -> Result<()> {
        self::execute::done(context, &self.connection, task).await
    }
}

#[async_trait::async_trait]
impl TaskSourceBackend for SQLiteTasks {
    async fn next(&mut self, context: &Context) -> Result<ReceivedTask> {
        loop {
            let next = self::execute::next(context, &self.connection, &self.subscriptions).await?;
            match next {
                Some(task) => return Ok(task),
                None => tokio::time::sleep(self.poll_delay).await,
            }
        }
    }

    async fn subscribe(&mut self, _: &Context, queue: &'static Queue) -> Result<()> {
        self.subscriptions.insert(&queue.queue, queue);
        Ok(())
    }
}

#[async_trait::async_trait]
impl TasksBackend for SQLiteTasks {
    async fn submit(&self, context: &Context, task: TaskSubmission) -> Result<()> {
        self::submit::submit(context, &self.connection, task).await
    }
}

#[cfg(test)]
mod tests {
    use super::SQLiteTasks;
    use crate::factory::create_client;

    /// Initialise an [`SQLiteTasks`] instance for unit tests.
    pub async fn sqlite_tasks() -> SQLiteTasks {
        let context = replicore_context::Context::fixture();
        let conf = crate::Conf::new(crate::factory::MEMORY_PATH);
        let connection = create_client(&context, &conf).await.unwrap();
        connection
            .call(move |connection| {
                crate::schema::migrations::runner()
                    .set_migration_table_name(crate::factory::REFINERY_SCHEMA_TABLE_NAME)
                    .run(connection)
                    .unwrap();
                Ok(())
            })
            .await
            .unwrap();
        SQLiteTasks::new(connection, &conf)
    }
}
