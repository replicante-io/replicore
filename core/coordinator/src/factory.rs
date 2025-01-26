//! Factory definitions needed to initialise [`Lease`]s.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use crate::Lease;

/// Interface to create a lease registry.
#[async_trait::async_trait]
pub trait ILeaseFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Create an [`ILeaseFactory`] configured appropriately.
    async fn registry(&self, context: &Context, conf: &Json) -> Result<LeaseRegistry>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Synchronise (initialise or migrate) the coordinator backed.
    async fn sync<'a>(&self, args: LeaseFactorySyncArgs<'a>) -> Result<()>;
}

/// Interface to create new leases.
#[async_trait::async_trait]
pub trait ILeaseRegistry {
    /// Create a [`Lease`] object with the correct backend.
    async fn lease<'a>(&self, args: LeaseRegistryArgs<'a>) -> Result<Lease>;
}

/// Initialisation logic to obtain [`LeaseRegistry`]s.
#[derive(Clone)]
pub struct LeaseFactory {
    /// Inner factory implementation.
    inner: Arc<dyn ILeaseFactory>,
}

impl<F> From<F> for LeaseFactory
where
    F: ILeaseFactory + 'static,
{
    fn from(value: F) -> Self {
        let inner = Arc::new(value);
        LeaseFactory { inner }
    }
}

impl LeaseFactory {
    /// Validate the user provided configuration for the backend.
    pub fn conf_check(&self, context: &Context, conf: &Json) -> Result<()> {
        self.inner.conf_check(context, conf)
    }

    /// Create an [`ILeaseFactory`] configured appropriately.
    pub async fn registry(&self, context: &Context, conf: &Json) -> Result<LeaseRegistry> {
        self.inner.registry(context, conf).await
    }

    /// Register backend specific metrics.
    pub fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()> {
        self.inner.register_metrics(registry)
    }

    /// Synchronise (initialise or migrate) the coordinator backed.
    pub async fn sync(&self, context: &Context, conf: &Json) -> Result<()> {
        let args = LeaseFactorySyncArgs { conf, context };
        self.inner.sync(args).await
    }
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
        let args = LeaseRegistryArgs {
            context,
            lease_id: lease_id.into(),
        };
        self.inner.lease(args).await
    }
}

/// Arguments passed to to the [`ILeaseFactory::lease`] method.
pub struct LeaseRegistryArgs<'a> {
    /// Container for operation scoped values.
    pub context: &'a Context,

    /// ID of the lease to create.
    pub lease_id: String,
}
