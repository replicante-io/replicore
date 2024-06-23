//! In memory approximate view of a cluster for logic across an entire distributed cluster.
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::StoreExtras;
use replisdk::core::models::oaction::OAction;

use replicore_context::Context;
use replicore_store::Store;

mod builder;
mod load;
mod serialise;

pub mod errors;
pub use self::builder::ClusterViewBuilder;

/// In memory approximate view of a cluster for logic across an entire distributed cluster.
#[derive(Debug)]
pub struct ClusterView {
    /// Discovery record for the cluster.
    pub discovery: ClusterDiscovery,

    /// All known nodes in the cluster, indexed by node ID.
    pub nodes: HashMap<String, Arc<Node>>,

    /// Unfinished orchestrator actions for the cluster, indexed by action ID.
    pub oactions_unfinished: Vec<Arc<OAction>>,

    /// Cluster Specification record for the cluster.
    pub spec: ClusterSpec,

    /// Store-requiring extra information about nodes.
    pub store_extras: HashMap<String, Arc<StoreExtras>>,
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
        self::load::overall(&mut builder, context, store).await?;
        self::load::nodes(&mut builder, context, store).await?;
        self::load::oactions(&mut builder, context, store).await?;
        Ok(builder)
    }

    /// Create a [`ClusterViewBuilder`] initialised with basic information from this view.
    pub fn new_build(&self) -> Result<ClusterViewBuilder> {
        let mut cluster_new = Self::builder(self.spec.clone());
        cluster_new.discovery(self.discovery.clone())?;
        Ok(cluster_new)
    }
}
