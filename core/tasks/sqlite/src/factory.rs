//! Initialise SQLite Tasks backend.
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context as AnyContext;
use anyhow::Result;
use serde_json::Value as Json;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_tasks::execute::TaskAck;
use replicore_tasks::execute::TaskSource;
use replicore_tasks::factory::TasksFactory;
use replicore_tasks::factory::TasksFactoryArgs;
use replicore_tasks::factory::TasksFactorySyncArgs;
use replicore_tasks::submit::Tasks;

use crate::Conf;
use crate::ConfError;

/// Special path requesting the use of an in-memory tasks store.
pub const MEMORY_PATH: &str = ":memory:";

/// Name of the table to store refinery migration metadata into.
pub const REFINERY_SCHEMA_TABLE_NAME: &str = "refinery_schema_history__tasks";

/// Initialise SQLite Tasks backend.
pub struct SQLiteFactory;

#[async_trait::async_trait]
impl TasksFactory for SQLiteFactory {
    fn conf_check(&self, _: &Context, conf: &Json) -> Result<()> {
        serde_json::from_value::<Conf>(conf.clone()).context(ConfError)?;
        Ok(())
    }

    async fn consume<'a>(&self, args: TasksFactoryArgs<'a>) -> Result<(TaskSource, TaskAck)> {
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = create_client(args.context, &conf).await?;
        let ack = crate::statements::SQLiteTasks::new(client, &conf);
        let source = ack.clone();
        Ok((TaskSource::from(source), TaskAck::from(ack)))
    }

    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()> {
        crate::telemetry::register_metrics(registry)?;
        Ok(())
    }

    async fn submit<'a>(&self, args: TasksFactoryArgs<'a>) -> Result<Tasks> {
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = create_client(args.context, &conf).await?;
        let tasks = crate::statements::SQLiteTasks::new(client, &conf);
        Ok(Tasks::from(tasks))
    }

    async fn sync<'a>(&self, args: TasksFactorySyncArgs<'a>) -> Result<()> {
        // Create the SQLite client.
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = create_client(args.context, &conf).await?;

        // Run migrations to ensure the DB is ready for use.
        let init_error: Arc<Mutex<Option<refinery::Error>>> = Default::default();
        let init_error_inner = Arc::clone(&init_error);
        client
            .call(move |connection| {
                let result = crate::schema::migrations::runner()
                    .set_migration_table_name(REFINERY_SCHEMA_TABLE_NAME)
                    .run(connection);
                if let Err(error) = result {
                    init_error_inner
                        .lock()
                        .expect("SQLiteStore sync error lock poisoned")
                        .replace(error);
                }
                Ok(())
            })
            .await?;

        // Extract the initialisation error, if any.
        let error = init_error
            .lock()
            .expect("SQLiteStore sync error lock poisoned")
            .take();
        if let Some(error) = error {
            return Err(error.into());
        }
        Ok(())
    }
}

/// Create a SQLite DB [`Connection`] to store and retrieve tasks.
///
/// The special [`MEMORY_PATH`] constant can be specified to create an in-memory store.
///
/// NOTE:
///   The use of an in-memory store is only intended for tests and experimentation
///   as all data will be lost as soon as the process terminates.
pub(crate) async fn create_client(context: &Context, conf: &Conf) -> Result<Connection> {
    // Open or create the SQLite DB.
    let path = &conf.path;
    let connection = if path == MEMORY_PATH {
        slog::warn!(
            context.logger,
            "Using in-memory store means data will be lost once the process terminates"
        );
        Connection::open_in_memory().await
    } else {
        Connection::open(path).await
    };
    let connection = connection?;
    Ok(connection)
}
