//! Coordination Leases backed by a SQLite DB.
use std::time::Duration;

use anyhow::Context as AnyContext;
use anyhow::Result;
use serde_json::Value as Json;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_coordinator::ILeaseRegistry;
use replicore_coordinator::Lease;
use replicore_coordinator::LeaseFactory;
use replicore_coordinator::LeaseFactorySyncArgs;
use replicore_coordinator::LeaseRegistry;
use replicore_coordinator::LeaseRegistryArgs;

mod conf;
mod factory;
mod lease;
mod schema;
mod statements;
mod telemetry;

use self::conf::Conf;
use self::conf::ConfError;

/// Factory for SQLite backed [`LeaseRegistry`]s.
pub struct Factory;

#[async_trait::async_trait]
impl LeaseFactory for Factory {
    fn conf_check(&self, _: &Context, conf: &Json) -> Result<()> {
        serde_json::from_value::<Conf>(conf.clone()).context(ConfError)?;
        Ok(())
    }

    async fn registry(&self, context: &Context, conf: &Json) -> Result<LeaseRegistry> {
        let conf: Conf = serde_json::from_value(conf.clone()).unwrap();
        let connection = crate::factory::create_client(context, &conf.path).await?;
        let ttl = Duration::from_secs(conf.ttl);
        let watch_interval = Duration::from_secs(conf.watch_interval);
        let reg = Registry {
            connection,
            ttl,
            watch_interval,
        };
        Ok(LeaseRegistry::from(reg))
    }

    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()> {
        crate::telemetry::register_metrics(registry)
    }

    async fn sync<'a>(&self, args: LeaseFactorySyncArgs<'a>) -> Result<()> {
        // Create the SQLite client.
        let conf: Conf = serde_json::from_value(args.conf.clone()).unwrap();
        let client = crate::factory::create_client(args.context, &conf.path).await?;

        // Run migrations to ensure the DB is ready for use.
        client
            .call(move |connection| {
                crate::schema::migrations::runner()
                    .set_migration_table_name(crate::factory::REFINERY_SCHEMA_TABLE_NAME)
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

/// Factory for SQLite backed [`Lease`]s.
pub struct Registry {
    /// SQLite DB connection backing [`Lease`]s.
    connection: Connection,

    /// Time to live, in seconds, before leases are either renewed or lost.
    ttl: Duration,

    /// Interval between DB checks and renewal attempts.
    watch_interval: Duration,
}

#[async_trait::async_trait]
impl ILeaseRegistry for Registry {
    async fn lease<'a>(&self, args: LeaseRegistryArgs<'a>) -> Result<Lease> {
        let connection = self.connection.clone();
        let backend = self::lease::LeaseBackend::new(
            &args.lease_id,
            self.ttl,
            self.watch_interval,
            connection,
        );
        let lease = Lease::new(args.context.clone(), args.lease_id, backend);
        Ok(lease)
    }

    async fn maintenance(&self, context: &Context) -> Result<()> {
        crate::statements::maintenance(context, &self.connection).await
    }
}
