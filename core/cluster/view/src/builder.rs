//! Incrementally build [`ClusterView`] instances.
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::naction::NAction;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::Shard;
use replisdk::core::models::node::StoreExtras;
use replisdk::core::models::oaction::OAction;

use crate::ClusterView;

macro_rules! check_cluster {
    ($builder: expr, $actual: expr) => {
        if $builder.spec.ns_id != $actual.ns_id || $builder.spec.cluster_id != $actual.cluster_id {
            anyhow::bail!(crate::errors::ClusterNotMatch {
                actual_cluster: $actual.cluster_id,
                actual_ns: $actual.ns_id,
                expect_cluster: $builder.spec.cluster_id.clone(),
                expect_ns: $builder.spec.ns_id.clone(),
            })
        }
    };
}

/// Incrementally build [`ClusterView`] instances.
pub struct ClusterViewBuilder {
    cluster: ClusterView,
}

impl ClusterViewBuilder {
    /// Return the cluster ID the view is about.
    pub fn cluster_id(&self) -> &str {
        &self.cluster.spec.cluster_id
    }

    /// Update the view with the given discovery record.
    ///
    /// The discovery is checked to make sure it references the same cluster (namespace and ID).
    pub fn discovery(&mut self, discovery: ClusterDiscovery) -> Result<&mut Self> {
        check_cluster!(self.cluster, discovery);
        self.cluster.discovery = discovery;
        Ok(self)
    }

    /// Complete the building process and return a [`ClusterView`].
    pub fn finish(self) -> ClusterView {
        self.cluster
    }

    /// Initialise a builder for an empty cluster.
    pub(crate) fn new(spec: ClusterSpec) -> ClusterViewBuilder {
        let cluster = ClusterView {
            discovery: ClusterDiscovery {
                ns_id: spec.ns_id.clone(),
                cluster_id: spec.cluster_id.clone(),
                nodes: Default::default(),
            },
            nactions_by_node: Default::default(),
            nodes: Default::default(),
            oactions_unfinished: Default::default(),
            spec,
            shards: Default::default(),
            store_extras: Default::default(),
            index_nactions_by_id: Default::default(),
        };
        ClusterViewBuilder { cluster }
    }

    /// Update the view with the given [`NAction`] record.
    pub fn naction(&mut self, action: NAction) -> Result<&mut Self> {
        check_cluster!(self.cluster, action);
        if action.state.phase.is_final() {
            anyhow::bail!(crate::errors::FinishedNAction {
                ns_id: action.ns_id,
                cluster_id: action.cluster_id,
                node_id: action.node_id,
                action_id: action.action_id,
            })
        }

        let action = Arc::new(action);
        let action_id = action.action_id;
        let node_id = action.node_id.clone();
        self.cluster
            .nactions_by_node
            .entry(node_id)
            .or_default()
            .push(action.clone());
        self.cluster.index_nactions_by_id.insert(action_id, action);
        Ok(self)
    }

    /// Update the view with the given node record.
    pub fn node_info(&mut self, node: Node) -> Result<&mut Self> {
        check_cluster!(self.cluster, node);
        let node_id = node.node_id.clone();
        self.cluster.nodes.insert(node_id, Arc::new(node));
        Ok(self)
    }

    /// Return the namespace ID the cluster belongs to.
    pub fn ns_id(&self) -> &str {
        &self.cluster.spec.ns_id
    }

    /// Include an unfinished orchestrator action into the view.
    pub fn oaction(&mut self, oaction: OAction) -> Result<&mut Self> {
        check_cluster!(self.cluster, oaction);
        if oaction.state.is_final() {
            anyhow::bail!(crate::errors::FinishedOAction {
                ns_id: oaction.ns_id,
                cluster_id: oaction.cluster_id,
                action_id: oaction.action_id,
            })
        }

        let oaction = Arc::new(oaction);
        self.cluster.oactions_unfinished.push(oaction);
        Ok(self)
    }

    /// Remvoe a [`NAction`] record from the cluster view following it reaching a final state.
    pub fn remove_naction(&mut self, action: &NAction) -> Result<&mut Self> {
        if !action.state.phase.is_final() {
            anyhow::bail!(crate::errors::UnfinishedNAction {
                ns_id: action.ns_id.clone(),
                cluster_id: action.cluster_id.clone(),
                node_id: action.node_id.clone(),
                action_id: action.action_id,
            })
        }

        // Remove the action from all views.
        self.cluster.index_nactions_by_id.remove(&action.action_id);

        // Remove the action from the scheduling list.
        let actions = self.cluster.nactions_by_node.get_mut(&action.node_id);
        if let Some(actions) = actions {
            let index = actions
                .iter()
                .position(|entry| entry.action_id == action.action_id);
            if let Some(index) = index {
                actions.remove(index);
            }
        }
        Ok(self)
    }

    /// Update the view with the given [`Shard`] record.
    pub fn shard(&mut self, shard: Shard) -> Result<&mut Self> {
        check_cluster!(self.cluster, shard);
        let node_id = shard.node_id.clone();
        let shard_id = shard.shard_id.clone();
        self.cluster
            .shards
            .entry(node_id)
            .or_default()
            .insert(shard_id, Arc::new(shard));
        Ok(self)
    }

    /// Update the view with the given [`StoreExtra`] record.
    pub fn store_extras(&mut self, extras: StoreExtras) -> Result<&mut Self> {
        check_cluster!(self.cluster, extras);
        let node_id = extras.node_id.clone();
        self.cluster.store_extras.insert(node_id, Arc::new(extras));
        Ok(self)
    }

    /// Update a [`NAction`] record in the cluster view.
    pub fn update_naction(&mut self, action: NAction) -> Result<&mut Self> {
        check_cluster!(self.cluster, action);
        if action.state.phase.is_final() {
            anyhow::bail!(crate::errors::FinishedNAction {
                ns_id: action.ns_id,
                cluster_id: action.cluster_id,
                node_id: action.node_id,
                action_id: action.action_id,
            })
        }
        let action = Arc::new(action);
        let node_id = action.node_id.clone();

        // Replace the action in all views.
        self.cluster
            .index_nactions_by_id
            .insert(action.action_id, action.clone());

        // Replace action while presrving the correct place in the scheduling order.
        let actions = self.cluster.nactions_by_node.entry(node_id).or_default();
        let entry = actions
            .iter_mut()
            .find(|entry| entry.action_id == action.action_id);
        if let Some(entry) = entry {
            *entry = action;
        }
        Ok(self)
    }
}

impl std::ops::Deref for ClusterViewBuilder {
    type Target = ClusterView;
    fn deref(&self) -> &Self::Target {
        &self.cluster
    }
}

// TODO: unit test remove_naction
// TODO: unit test update_naction
