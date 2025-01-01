//! Factory definitions needed to initialise [`Lease`]s.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use crate::Lease;

/// Interface to logic that creates new leases.
#[async_trait::async_trait]
pub trait ILeaseFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Create a [`Lease`] object with the correct backend.
    async fn lease<'a>(&self, args: LeaseFactoryArgs<'a>) -> Result<Lease>;
}

/// Options for creation of a [`Lease`] object.
pub struct LeaseConf {
    /// Lease identifier so multiple process can coordinate access to the same resource.
    pub id: String,
}

/// Initialisation logic to obtain [`Lease`]s.
#[derive(Clone)]
pub struct LeaseFactory {
    /// Inner lease factory implementation.
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
    /// Create a [`Lease`] object with the correct backend.
    ///
    /// Note the lease will begin operating immediately after it is created.
    /// This includes attempting to acquire ownership and generating [`Lease::watch`] events.
    pub async fn get(&self, context: &Context, conf: &LeaseConf) -> Result<Lease> {
        let args = LeaseFactoryArgs { conf, context };
        self.inner.lease(args).await
    }
}

/// Arguments passed to to the [`ILeaseFactory::lease`] method.
pub struct LeaseFactoryArgs<'a> {
    /// Configuration of the lease to create.
    pub conf: &'a LeaseConf,

    /// Container for operation scoped values.
    pub context: &'a Context,
}
