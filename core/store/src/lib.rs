//! Persistent storage interface for RepliCore Control Plane.
//!
//! ## An ergonomic interface
//!
//! The objective of the [`Store`] API is to provide the nicest interface
//! I could possibly think of while preserving the ability of [`StoreBackend`]s to pick
//! the most efficient implementation they can create.
//!
//! To achieve this:
//!
//! - The [`Store`] interface focuses on high level operations.
//! - To make the interface intuitive operations are grouped into a small set of
//!   methods that except different data and return different data.
//! - This is implemented with a combination of an internal (sealed) `trait` and enums.
//!
//! For example to delete a namespace.
//!
//! ```ignore
//! use replisdk::core::models::namespace::Namespace;
//! use replicore_store::delete::DeleteNamespace;
//!
//! // Delete a namespace for which you have a model.
//! let namespace = Namespace { id: String::from("my-namespace") };
//! store.delete(context, namespace).await?;
//!
//! // Delete a namespace for which you have the ID.
//! let namespace = DeleteNamespace::from("my-namespace");
//! store.delete(context, namespace).await?;
//! ```
//!
//! ### Backend implementations
//!
//! Backend implementations receive a wrapping `enum` type for the operation group to implement.
//! This makes adding new operations a simpler, with less files needing to change.
//!
//! The cost of this approach is that backend implementation need to deal with these type enums
//! and ensure the returned type matches what the requested operation expects.
//! If you fail to properly do this the [`Store`] interface will panic while converting types.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

pub mod delete;
pub mod ids;
pub mod persist;
pub mod query;

#[cfg(any(test, feature = "test-fixture"))]
mod fixture;
#[cfg(any(test, feature = "test-fixture"))]
pub use self::fixture::StoreFixture;

#[cfg(test)]
mod tests;

use self::delete::DeleteOp;
use self::delete::DeleteOps;
use self::delete::DeleteResponses;
use self::persist::PersistOp;
use self::persist::PersistOps;
use self::persist::PersistResponses;
use self::query::QueryOp;
use self::query::QueryOps;
use self::query::QueryResponses;

/// Query, persist and manipulate Control Plane state with a database.
#[derive(Clone)]
pub struct Store {
    /// Runtime configured implementation of the persistent store.
    inner: Arc<dyn StoreBackend>,
}

impl Store {
    /// Delete individual records from the persistent store.
    pub async fn delete<O>(&self, context: &Context, op: O) -> Result<O::Response>
    where
        O: DeleteOp,
    {
        let op: DeleteOps = op.into();
        let response = self.inner.delete(context, op).await;
        response.map(O::Response::from)
    }

    /// Query records from the persistent store.
    pub async fn query<O>(&self, context: &Context, op: O) -> Result<O::Response>
    where
        O: QueryOp,
    {
        let op: QueryOps = op.into();
        let response = self.inner.query(context, op).await;
        response.map(O::Response::from)
    }

    /// Persist records from the persistent store.
    pub async fn persist<O>(&self, context: &Context, op: O) -> Result<O::Response>
    where
        O: PersistOp,
    {
        let op: PersistOps = op.into();
        let response = self.inner.persist(context, op).await;
        response.map(O::Response::from)
    }
}

impl<T> From<T> for Store
where
    T: StoreBackend + 'static,
{
    fn from(value: T) -> Self {
        Store {
            inner: Arc::new(value),
        }
    }
}

#[cfg(any(test, feature = "test-fixture"))]
impl Store {
    /// Initialise a new store backend fixture for unit tests.
    pub fn fixture() -> Self {
        let inner = StoreFixture::default();
        Self::from(inner)
    }
}

/// Operations implemented by Persistent Stores supported by Replicante Core.
#[async_trait::async_trait]
pub trait StoreBackend: Send + Sync {
    /// Delete individual records from the persistent store.
    async fn delete(&self, context: &Context, op: DeleteOps) -> Result<DeleteResponses>;

    /// Query records from the persistent store.
    async fn query(&self, context: &Context, op: QueryOps) -> Result<QueryResponses>;

    /// Persist records from the persistent store.
    async fn persist(&self, context: &Context, op: PersistOps) -> Result<PersistResponses>;
}

/// Initialisation logic for the Persistent Store and the client to access it.
#[async_trait::async_trait]
pub trait StoreFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Instantiate a [`Store`] object to access persistent state.
    async fn store<'a>(&self, args: StoreFactoryArgs<'a>) -> Result<Store>;

    /// Synchronise (initialise or migrate) the Persistent store to handle [`Store`] operations.
    async fn sync<'a>(&self, args: StoreFactorySyncArgs<'a>) -> Result<()>;
}

/// Arguments passed to the [`StoreFactory`] client initialisation method.
pub struct StoreFactoryArgs<'a> {
    /// The configuration block for the backend to initialise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}

/// Arguments passed to the [`StoreFactory`] client synchronisation method.
pub struct StoreFactorySyncArgs<'a> {
    /// The configuration block for the backend to synchronise.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}
