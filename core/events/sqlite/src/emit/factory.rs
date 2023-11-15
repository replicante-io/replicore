//! Factory for the SQLite Events backend.
use anyhow::Context as AnyContext;
use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;
use replicore_events::emit::Events;
use replicore_events::emit::EventsFactory;
use replicore_events::emit::EventsFactoryArgs;
use replicore_events::emit::EventsFactorySyncArgs;

use super::events::SQLiteEvents;
use crate::Conf;

/// The SQLite events backend configuration is not valid.
#[derive(Debug, thiserror::Error)]
#[error("the SQLite events backend configuration is not valid")]
pub struct ConfError;

/// Initialise SQLite Events streams.
pub struct SQLiteFactory;

#[async_trait::async_trait]
impl EventsFactory for SQLiteFactory {
    fn conf_check(&self, _context: &Context, conf: &Json) -> Result<()> {
        serde_json::from_value::<Conf>(conf.clone()).context(ConfError)?;
        Ok(())
    }

    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()> {
        crate::telemetry::register_metrics(registry)
    }

    async fn events<'a>(&self, args: EventsFactoryArgs<'a>) -> Result<Events> {
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let events = SQLiteEvents::new(args.context, &conf).await?;
        Ok(Events::from(events))
    }

    async fn sync<'a>(&self, args: EventsFactorySyncArgs<'a>) -> Result<()> {
        // Create the SQLite client.
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = crate::client::create(args.context, &conf.path).await?;

        // Run migrations to ensure the DB is ready for use.
        client
            .call(|connection| {
                crate::schema::migrations::runner()
                    .set_migration_table_name(crate::client::REFINERY_SCHEMA_TABLE_NAME)
                    .run(connection)
                    .map_err(|error| {
                        let error = Box::new(error);
                        tokio_rusqlite::Error::Other(error)
                    })
            })
            .await?;
        Ok(())
    }
}
