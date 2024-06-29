//! In memory approximate view of a cluster for logic across an entire distributed cluster.
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::naction::NAction;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::Shard;
use replisdk::core::models::node::StoreExtras;
use replisdk::core::models::oaction::OAction;

use replicore_context::Context;
use replicore_store::Store;

mod builder;
mod load;
mod serialise;

pub mod errors;
pub use self::builder::ClusterViewBuilder;

/// Nested index for the nactions collection, indexed by action ID.
pub type NodeActions = HashMap<Uuid, Arc<NAction>>;

/// Nested index for the shards collection, indexed by shard ID.
pub type NodeShards = HashMap<String, Arc<Shard>>;

/// In memory approximate view of a cluster for logic across an entire distributed cluster.
#[derive(Debug)]
pub struct ClusterView {
    /// Discovery record for the cluster.
    pub discovery: ClusterDiscovery,

    /// Unfinished node actions for the cluster, indexed by node ID and action ID.
    pub nactions_by_node: HashMap<String, NodeActions>,

    /// All known nodes in the cluster, indexed by node ID.
    pub nodes: HashMap<String, Arc<Node>>,

    /// Unfinished orchestrator actions for the cluster, ordered by creation time.
    pub oactions_unfinished: Vec<Arc<OAction>>,

    /// Cluster Specification record for the cluster.
    pub spec: ClusterSpec,

    /// Information about shards found on cluster nodes.
    pub shards: HashMap<String, NodeShards>,

    /// Store-requiring extra information about nodes.
    pub store_extras: HashMap<String, Arc<StoreExtras>>,

    // --- Indexes to efficiently access cluster entries with secondaty patterns ---
    /// Access node actions by action ID.
    pub index_nactions_by_id: HashMap<Uuid, Arc<NAction>>,
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
        self::load::nactions(&mut builder, context, store).await?;
        self::load::oactions(&mut builder, context, store).await?;
        Ok(builder)
    }

    /// Lookup a node action across all nodes.
    pub fn lookup_node_action(&self, action_id: &Uuid) -> Option<&NAction> {
        self.index_nactions_by_id.get(action_id).map(AsRef::as_ref)
    }

    /// Create a [`ClusterViewBuilder`] initialised with basic information from this view.
    pub fn new_build(&self) -> Result<ClusterViewBuilder> {
        let mut cluster_new = Self::builder(self.spec.clone());
        cluster_new.discovery(self.discovery.clone())?;
        Ok(cluster_new)
    }

    /// Set of all unfinished node action IDs across all nodes.
    pub fn unfinished_node_actions(&self) -> HashSet<Uuid> {
        self.index_nactions_by_id.keys().copied().collect()
    }
}
