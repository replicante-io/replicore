//! Initialise SQLite Persistent Store.
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context as AnyContext;
use anyhow::Result;
use serde_json::Value as Json;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_store::Store;
use replicore_store::StoreFactory;
use replicore_store::StoreFactoryArgs;
use replicore_store::StoreFactorySyncArgs;

use crate::Conf;
use crate::ConfError;

/// Special path requesting the use of an in-memory store.
pub const MEMORY_PATH: &str = ":memory:";

/// Name of the table to store refinery migration metadata into.
pub const REFINERY_SCHEMA_TABLE_NAME: &str = "refinery_schema_history__store";

/// Initialise SQLite Persistent Store.
pub struct SQLiteFactory;

#[async_trait::async_trait]
impl StoreFactory for SQLiteFactory {
    fn conf_check(&self, _: &Context, conf: &Json) -> Result<()> {
        serde_json::from_value::<Conf>(conf.clone()).context(ConfError)?;
        Ok(())
    }

    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()> {
        crate::telemetry::register_metrics(registry)
    }

    async fn store<'a>(&self, args: StoreFactoryArgs<'a>) -> Result<Store> {
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = create_client(args.context, &conf.path).await?;
        let store = crate::statements::SQLiteStore::new(client);
        Ok(Store::from(store))
    }

    async fn sync<'a>(&self, args: StoreFactorySyncArgs<'a>) -> Result<()> {
        // Create the SQLite client.
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = create_client(args.context, &conf.path).await?;

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

/// Create a SQLite DB [`Connection`] to the persistent store.
///
/// The special [`MEMORY_PATH`] constant can be specified to create an in-memory store.
///
/// NOTE:
///   The use of an in-memory store is only intended for tests and experimentation
///   as all data will be lost as soon as the process terminates.
pub(crate) async fn create_client(context: &Context, path: &str) -> Result<Connection> {
    // Open or create the SQLite DB.
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
