//! In memory approximate view of a cluster for logic across an entire distributed cluster.
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::oaction::OAction;

use replicore_context::Context;
use replicore_store::query::LookupClusterDiscovery;
use replicore_store::Store;

mod builder;

pub mod errors;
pub use self::builder::ClusterViewBuilder;

/// In memory approximate view of a cluster for logic across an entire distributed cluster.
#[derive(Debug)]
pub struct ClusterView {
    /// Discovery record for the cluster.
    pub discovery: ClusterDiscovery,

    /// Unfinished orchestrator actions for the cluster.
    pub oactions_unfinished: Vec<Arc<OAction>>,

    /// Cluster Specification record for the cluster.
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

        // Load overall cluster information.
        let op = LookupClusterDiscovery::by(builder.ns_id(), builder.cluster_id());
        if let Some(discovery) = store.query(context, op).await? {
            builder.discovery(discovery)?;
        }

        // Load orchestrator action information.
        // TODO: load unfinished orchestrator actions.

        Ok(builder)
    }
}
