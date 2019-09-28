use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_service_healthcheck::HealthChecks;

use crate::backend::backend_factory;
use crate::backend::StoreImpl;
use crate::Config;
use crate::Result;

pub mod actions;
pub mod agent;
pub mod agents;
pub mod cluster;
pub mod legacy;
pub mod node;
pub mod nodes;
pub mod persist;
pub mod shard;
pub mod shards;

use self::actions::Actions;
use self::agent::Agent;
use self::agents::Agents;
use self::cluster::Cluster;
use self::legacy::Legacy;
use self::node::Node;
use self::nodes::Nodes;
use self::persist::Persist;
use self::shard::Shard;
use self::shards::Shards;

/// Interface to Replicante primary store layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
///
/// # Purpose
/// The primary store is responsable for data used by Replicante Core itself and
/// needed to implement the platform.
///
/// Any other data, such as historical or aggregated data kept purely for the API,
/// debugging, introspection, and similar should be stored elsewhere.
///
/// # Concurrency and transactions
/// The store does not provide a transactional interface.
/// Concurrency is allowed by sharding, with processes relying on the coordinator to avoid
/// stepping over each others toes.
///
/// The non-transactional, distributed, nature of a cluster state limits the value
/// of transactions when it comes to requirements around the cluster state.
#[derive(Clone)]
pub struct Store {
    store: StoreImpl,
}

impl Store {
    /// Instantiate a new storage interface.
    pub fn make<T>(
        config: Config,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Store>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let store = backend_factory(config, logger, healthchecks, tracer)?;
        Ok(Store { store })
    }

    /// Instantiate a store with the given implementation.
    #[cfg(feature = "with_test_support")]
    pub(crate) fn with_impl(store: StoreImpl) -> Store {
        Store { store }
    }

    /// Operate on actions for the cluster identified by cluster_id.
    pub fn actions(&self, cluster_id: String) -> Actions {
        let actions = self.store.actions();
        let attrs = self::actions::ActionsAttributes { cluster_id };
        Actions::new(actions, attrs)
    }

    /// Operate on the agent identified by the provided cluster_id and host.
    pub fn agent(&self, cluster_id: String, host: String) -> Agent {
        let agent = self.store.agent();
        let attrs = self::agent::AgentAttribures { cluster_id, host };
        Agent::new(agent, attrs)
    }

    /// Operate on all agent in the cluster identified by cluster_id.
    pub fn agents(&self, cluster_id: String) -> Agents {
        let agents = self.store.agents();
        let attrs = self::agents::AgentsAttribures { cluster_id };
        Agents::new(agents, attrs)
    }

    /// Operate on cluster-level models.
    pub fn cluster(&self, cluster_id: String) -> Cluster {
        let cluster = self.store.cluster();
        let attrs = self::cluster::ClusterAttribures { cluster_id };
        Cluster::new(cluster, attrs)
    }

    /// Legacy operations that need to be moved to other crates.
    pub fn legacy(&self) -> Legacy {
        let legacy = self.store.legacy();
        Legacy::new(legacy)
    }

    /// Operate on the node identified by the provided cluster_id and node_id.
    pub fn node(&self, cluster_id: String, node_id: String) -> Node {
        let node = self.store.node();
        let attrs = self::node::NodeAttribures {
            cluster_id,
            node_id,
        };
        Node::new(node, attrs)
    }

    /// Operate on all nodes in the cluster identified by cluster_id.
    pub fn nodes(&self, cluster_id: String) -> Nodes {
        let nodes = self.store.nodes();
        let attrs = self::nodes::NodesAttribures { cluster_id };
        Nodes::new(nodes, attrs)
    }

    /// Persist (insert or update) models to the store.
    pub fn persist(&self) -> Persist {
        let persist = self.store.persist();
        Persist::new(persist)
    }

    /// Operate on the shard identified by the provided cluster_id, node_id, shard_id.
    pub fn shard(&self, cluster_id: String, node_id: String, shard_id: String) -> Shard {
        let shard = self.store.shard();
        let attrs = self::shard::ShardAttribures {
            cluster_id,
            node_id,
            shard_id,
        };
        Shard::new(shard, attrs)
    }

    /// Operate on all shards in the cluster identified by cluster_id.
    pub fn shards(&self, cluster_id: String) -> Shards {
        let shards = self.store.shards();
        let attrs = self::shards::ShardsAttribures { cluster_id };
        Shards::new(shards, attrs)
    }
}
