//! Factory definitions needed to initialise [`Lease`]s.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use crate::Lease;
use crate::LeaseBuilder;

/// Interface to create a lease registry.
#[async_trait::async_trait]
pub trait LeaseFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Create an [`LeaseFactory`] configured appropriately.
    async fn registry(&self, context: &Context, conf: &Json) -> Result<LeaseRegistry>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Synchronise (initialise or migrate) the coordinator backed.
    async fn sync<'a>(&self, args: LeaseFactorySyncArgs<'a>) -> Result<()>;
}

/// Interface to create new leases.
#[async_trait::async_trait]
pub trait ILeaseRegistry: Send + Sync {
    /// Create a [`LeaseBuilder`] object with the correct backend.
    async fn lease_builder<'a>(&self, args: LeaseRegistryArgs<'a>) -> Result<LeaseBuilder>;

    /// Execute backend-specific maintenance of the coordinator service.
    async fn maintenance(&self, context: &Context) -> Result<()>;
}

/// Arguments passed to the [`LeaseFactory`] client synchronisation method.
pub struct LeaseFactorySyncArgs<'a> {
    /// Configuration of the coordinator backend to create.
    pub conf: &'a Json,

    /// Container for operation scoped values.
    pub context: &'a Context,
}

/// Initialisation logic to obtain [`Lease`]s.
#[derive(Clone)]
pub struct LeaseRegistry {
    /// Inner lease factory implementation.
    inner: Arc<dyn ILeaseRegistry>,
}

impl<F> From<F> for LeaseRegistry
where
    F: ILeaseRegistry + 'static,
{
    fn from(value: F) -> Self {
        let inner = Arc::new(value);
        LeaseRegistry { inner }
    }
}

impl LeaseRegistry {
    /// Create a [`Lease`] object with the correct backend.
    ///
    /// Note the lease will begin operating immediately after it is created.
    /// This includes attempting to acquire ownership and generating [`Lease::watch`] events.
    pub async fn lease<S>(&self, context: &Context, lease_id: S) -> Result<Lease>
    where
        S: Into<String>,
    {
        let lease = self.lease_builder(context, lease_id).await?;
        Ok(lease.build())
    }

    /// Create a [`LeaseBuilder`] object with the correct backend.
    ///
    /// The lease will not begin operating until it is fully built with [`LeaseBuilder::build`].
    pub async fn lease_builder<S>(&self, context: &Context, lease_id: S) -> Result<LeaseBuilder>
    where
        S: Into<String>,
    {
        let args = LeaseRegistryArgs {
            context,
            lease_id: lease_id.into(),
        };
        self.inner.lease_builder(args).await
    }

    /// Execute backend-specific maintenance of the coordinator service.
    pub async fn maintenance(&self, context: &Context) -> Result<()> {
        self.inner.maintenance(context).await
    }
}

/// Arguments passed to to the [`LeaseFactory::lease`] method.
pub struct LeaseRegistryArgs<'a> {
    /// Container for operation scoped values.
    pub context: &'a Context,

    /// ID of the lease to create.
    pub lease_id: String,
}
