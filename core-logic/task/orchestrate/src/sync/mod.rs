//! Synchronise information about nodes with the control plane.
//!
//! Synchronisation processes each node individually to refresh the current state.
//! If nodes fail to Synchronise this is noted but the process carries on to prevent
//! individual nodes from blocking all cluster management.
//!
//! The sync process does NOT schedule new node actions, this is expected separately.
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context as AnyContext;
use anyhow::Result;

use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::node::Shard;
use replisdk::core::models::node::StoreExtras;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_cluster_models::OrchestrateMode;
use replicore_cluster_models::OrchestrateReport;
use replicore_cluster_models::OrchestrateReportNote;
use replicore_cluster_view::ClusterView;
use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;

mod error;
mod nactions;
mod node;
mod store;

use self::error::NodeSpecificCheck;
use self::error::NodeSpecificError;
use crate::init::InitData;

/// Data used in the sync phase of cluster orchestration.
pub struct SyncData {
    pub cluster_current: ClusterView,
    pub cluster_new: Mutex<ClusterViewBuilder>,
    pub injector: Injector,
    pub mode: OrchestrateMode,
    pub ns: Namespace,
    pub report: Mutex<OrchestrateReport>,
}

impl std::fmt::Debug for SyncData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncData")
            .field("cluster_current", &self.cluster_current)
            .field("cluster_new", &"ClusterViewBuilder { ... }")
            .field("injector", &"Injector { ... }")
            .field("mode", &self.mode)
            .field("ns", &self.ns)
            .field("report", &self.report)
            .finish()
    }
}

impl SyncData {
    /// Quick access to the Cluster ID being orchestrated.
    pub fn cluster_id(&self) -> &str {
        &self.cluster_current.spec.cluster_id
    }

    /// Mutable access to the new `ClusterView` builder.
    pub fn cluster_new_mut(&self) -> MutexGuard<ClusterViewBuilder> {
        self.cluster_new
            .lock()
            .expect("orchestrate task cluster_new lock poisoned")
    }

    /// Initialise the [`SyncData`] from [`InitData`].
    pub fn convert(data: InitData) -> Result<Self> {
        let cluster_new = data.cluster_current.new_build()?;
        let report = data.report();
        let data = Self {
            cluster_current: data.cluster_current,
            cluster_new: Mutex::new(cluster_new),
            injector: data.injector,
            mode: data.mode,
            ns: data.ns,
            report: Mutex::new(report),
        };
        Ok(data)
    }

    /// Mutable access to the [`OrchestrateReport`].
    pub fn report_mut(&self) -> MutexGuard<OrchestrateReport> {
        self.report
            .lock()
            .expect("orchestrate task report lock poisoned")
    }

    /// Quick access to the Namespace ID being orchestrate.
    pub fn ns_id(&self) -> &str {
        &self.cluster_current.spec.ns_id
    }
}

/// Synchronise information about nodes with the control plane.
pub async fn nodes(context: &Context, data: &SyncData) -> Result<()> {
    // Refresh the state of nodes in the discovery record.
    let mut current_nodes: HashSet<&String> = HashSet::new();
    for node in &data.cluster_current.discovery.nodes {
        current_nodes.insert(&node.node_id);
        let result = sync_node(context, data, node).await;
        let result = result.with_node_specific()?;
        if let Err(error) = result {
            let message = "Node processing interrupted early";
            let note = OrchestrateReportNote::error(message, error);
            data.report_mut().notes.push(note);
        }
    }

    // Delete records about nodes no longer reported.
    let nodes = data
        .cluster_current
        .nodes
        .values()
        .filter(|node| !current_nodes.contains(&node.node_id));
    for node in nodes {
        let event = Event::new_with_payload(crate::constants::NODE_DELETE, node.as_ref().clone())?;
        data.injector.events.change(context, event).await?;

        let node_id =
            replicore_store::ids::NodeID::by(&node.ns_id, &node.cluster_id, &node.node_id);
        let op = replicore_store::persist::NodeCancelAllActions::from(node_id.clone());
        data.injector.store.persist(context, op).await?;
        data.injector.store.delete(context, node_id).await?;
    }
    Ok(())
}

/// Sync the specified node in isolation.
async fn sync_node(context: &Context, data: &SyncData, node: &ClusterDiscoveryNode) -> Result<()> {
    // Create a client to interact with the node.
    let client = data
        .injector
        .clients
        .agent
        .factory(context, &data.cluster_current.spec, node)
        .await?;

    // Fetch essential node information we can't continue without.
    let node_info = self::node::unreachable(&data.cluster_current.spec, node);
    let ag_node = match client.info_node().await.context(NodeSpecificError) {
        Ok(node) => node,
        Err(error) => {
            self::node::persist(context, data, node_info).await?;
            return Err(error);
        }
    };

    // Fetch all other node information and process them as best as possible.
    let store_info = client.info_store().await.context(NodeSpecificError);
    let shards = client.info_shards().await.context(NodeSpecificError);

    // Process fetched information for node sync.
    let incomplete = store_info.is_err() || shards.is_err();
    let node_info = self::node::process(incomplete, ag_node, node_info);
    self::node::persist(context, data, node_info).await?;

    match store_info {
        Ok(store_info) => {
            let store_info = StoreExtras {
                ns_id: data.ns_id().to_string(),
                cluster_id: data.cluster_id().to_string(),
                node_id: node.node_id.clone(),
                attributes: store_info.attributes,
                fresh: true,
            };
            self::store::persist_extras(context, data, store_info).await?;
        }
        Err(error) => {
            let message = "Skipped sync of Store Info due to agent error";
            let mut note = OrchestrateReportNote::error(message, error);
            note.for_node(&node.node_id);
            data.report_mut().notes.push(note);
            self::store::stale_extras(context, data, node).await?;
        }
    }

    match shards {
        Ok(shards) => {
            let shards = shards
                .shards
                .into_iter()
                .map(|shard| Shard {
                    ns_id: data.ns_id().to_string(),
                    cluster_id: data.cluster_id().to_string(),
                    node_id: node.node_id.clone(),
                    shard_id: shard.shard_id,
                    commit_offset: shard.commit_offset,
                    fresh: true,
                    lag: shard.lag,
                    role: shard.role,
                })
                .collect();
            self::store::persist_shards(context, data, shards).await?;
        }
        Err(error) => {
            let message = "Skipped sync of Shards Info due to agent error";
            let mut note = OrchestrateReportNote::error(message, error);
            note.for_node(&node.node_id);
            self::store::stale_shards(context, data, node).await?;
        }
    }

    self::nactions::sync(context, data, node, &client).await
}
