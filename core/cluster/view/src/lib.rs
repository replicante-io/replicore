//! In memory approximate view of a cluster for logic across an entire distributed cluster.
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;

use replicore_context::Context;
use replicore_store::query::LookupClusterDiscovery;
use replicore_store::Store;

mod builder;

pub mod errors;
pub use self::builder::ClusterViewBuilder;

/// In memory approximate view of a cluster for logic across an entire distributed cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterView {
    pub discovery: ClusterDiscovery,
    pub spec: ClusterSpec,
}

impl ClusterView {
    /// Initialise an empty builder instance.
    pub fn builder(spec: ClusterSpec) -> ClusterViewBuilder {
        ClusterViewBuilder::new(spec)
    }

    /// Build a [`ClusterView`] with information loaded from the store.
    pub async fn load(
        context: &Context,
        store: &Store,
        spec: ClusterSpec,
    ) -> Result<ClusterViewBuilder> {
        let mut builder = Self::builder(spec);

        let op = LookupClusterDiscovery::by(builder.ns_id(), builder.cluster_id());
        if let Some(discovery) = store.query(context, op).await? {
            builder.discovery(discovery)?;
        }

        Ok(builder)
    }
}
