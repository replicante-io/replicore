//! SQL statements to implement the [`StoreBackend`] with SQLite.
use anyhow::Result;
use tokio_rusqlite::Connection;

use replicore_context::Context;
use replicore_store::delete::DeleteOps;
use replicore_store::delete::DeleteResponses;
use replicore_store::persist::PersistOps;
use replicore_store::persist::PersistResponses;
use replicore_store::query::QueryOps;
use replicore_store::query::QueryResponses;
use replicore_store::StoreBackend;

mod cluster_spec;
mod namespace;
mod platform;

/// implementation of the [`StoreBackend`] interface using SQLite.
pub struct SQLiteStore {
    /// Connection to the SQLite DB persisting data.
    connection: Connection,
}

impl SQLiteStore {
    /// Initialise a new SQLite backed [`StoreBackend`].
    pub fn new(connection: Connection) -> Self {
        SQLiteStore { connection }
    }
}

#[async_trait::async_trait]
impl StoreBackend for SQLiteStore {
    async fn delete(&self, context: &Context, op: DeleteOps) -> Result<DeleteResponses> {
        match op {
            DeleteOps::ClusterSpec(cluster) => {
                self::cluster_spec::delete(context, &self.connection, cluster)
                    .await
                    .map(|_| DeleteResponses::Success)
            }
            DeleteOps::Namespace(ns) => self::namespace::delete(context, &self.connection, ns)
                .await
                .map(|_| DeleteResponses::Success),
            DeleteOps::Platform(pl) => self::platform::delete(context, &self.connection, pl)
                .await
                .map(|_| DeleteResponses::Success),
        }
    }

    async fn query(&self, context: &Context, op: QueryOps) -> Result<QueryResponses> {
        match op {
            QueryOps::ClusterSpec(spec) => {
                let spec = self::cluster_spec::lookup(context, &self.connection, spec).await?;
                Ok(QueryResponses::ClusterSpec(spec))
            }
            QueryOps::ListClusterSpecIds(ns) => {
                let list = self::cluster_spec::list(context, &self.connection, ns).await?;
                Ok(QueryResponses::StringStream(list))
            }
            QueryOps::ListNamespaceIds => {
                let list = self::namespace::list(context, &self.connection).await?;
                Ok(QueryResponses::StringStream(list))
            }
            QueryOps::ListPlatformIds(ns) => {
                let list = self::platform::list(context, &self.connection, ns).await?;
                Ok(QueryResponses::StringStream(list))
            }
            QueryOps::Namespace(ns) => {
                let ns = self::namespace::lookup(context, &self.connection, ns).await?;
                Ok(QueryResponses::Namespace(ns))
            }
            QueryOps::Platform(pl) => {
                let pl = self::platform::lookup(context, &self.connection, pl).await?;
                Ok(QueryResponses::Platform(pl))
            }
        }
    }

    async fn persist(&self, context: &Context, op: PersistOps) -> Result<PersistResponses> {
        match op {
            PersistOps::ClusterSpec(spec) => {
                self::cluster_spec::persist(context, &self.connection, spec)
                    .await
                    .map(|_| PersistResponses::Success)
            }
            PersistOps::Namespace(ns) => self::namespace::persist(context, &self.connection, ns)
                .await
                .map(|_| PersistResponses::Success),
            PersistOps::Platform(pl) => self::platform::persist(context, &self.connection, pl)
                .await
                .map(|_| PersistResponses::Success),
        }
    }
}

#[cfg(test)]
mod tests {
    use replicore_store::Store;

    use super::SQLiteStore;
    use crate::factory::create_client;

    /// Initialise an [`SQLiteStore`] instance for unit tests.
    pub async fn sqlite_store() -> SQLiteStore {
        let context = replicore_context::Context::fixture();
        let connection = create_client(&context, crate::factory::MEMORY_PATH)
            .await
            .unwrap();
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
        SQLiteStore { connection }
    }

    /// Same as [`sqlite_store`] but returns a user facing [`Store`] object instead.
    pub async fn store() -> Store {
        let store = sqlite_store().await;
        Store::from(store)
    }
}
