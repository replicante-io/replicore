use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::Logger;
use uuid::Uuid;

use replicante_service_healthcheck::HealthChecks;
use replicore_cluster_view::ClusterView;

use crate::backend::backend_factory;
use crate::backend::StoreImpl;
use crate::error::AnyWrap;
use crate::Config;
use crate::ErrorKind;
use crate::Result;

pub mod action;
pub mod actions;
pub mod agent;
pub mod agents;
pub mod cluster;
pub mod discovery_settings;
pub mod global_search;
pub mod legacy;
pub mod namespace;
pub mod namespaces;
pub mod node;
pub mod nodes;
pub mod orchestrator_action;
pub mod orchestrator_actions;
pub mod persist;
pub mod platform;
pub mod platforms;
pub mod shard;
pub mod shards;

use self::action::Action;
use self::actions::Actions;
use self::agent::Agent;
use self::agents::Agents;
use self::cluster::Cluster;
use self::discovery_settings::DiscoverySettings;
use self::global_search::GlobalSearch;
use self::legacy::Legacy;
use self::namespace::Namespace;
use self::namespaces::Namespaces;
use self::node::Node;
use self::nodes::Nodes;
use self::orchestrator_action::OrchestratorAction;
use self::orchestrator_actions::OrchestratorActions;
use self::persist::Persist;
use self::platform::Platform;
use self::platforms::Platforms;
use self::shard::Shard;
use self::shards::Shards;

/// Interface to Replicante primary store layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
///
/// # Purpose
/// The primary store is responsible for data used by Replicante Core itself and
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

    /// Operate on the action identified by the provided cluster_id and action_id.
    pub fn action(&self, cluster_id: String, action_id: Uuid) -> Action {
        let action = self.store.action();
        let attrs = self::action::ActionAttributes {
            action_id,
            cluster_id,
        };
        Action::new(action, attrs)
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
        let attrs = self::agent::AgentAttributes { cluster_id, host };
        Agent::new(agent, attrs)
    }

    /// Operate on all agent in the cluster identified by cluster_id.
    pub fn agents(&self, cluster_id: String) -> Agents {
        let agents = self.store.agents();
        let attrs = self::agents::AgentsAttributes { cluster_id };
        Agents::new(agents, attrs)
    }

    /// Operate on cluster-level models.
    pub fn cluster(&self, namespace: String, cluster_id: String) -> Cluster {
        let cluster = self.store.cluster();
        let attrs = self::cluster::ClusterAttributes {
            cluster_id,
            namespace,
        };
        Cluster::new(cluster, attrs)
    }

    /// Operate on DiscoverySettings objects in a namespace.
    pub fn discovery_settings(&self, namespace: String) -> DiscoverySettings {
        let discovery_settings = self.store.discovery_settings();
        let attrs = self::discovery_settings::DiscoverySettingsAttributes { namespace };
        DiscoverySettings::new(discovery_settings, attrs)
    }

    /// Search for specific records across the entrie system (no namespaces, clusters, ...).
    pub fn global_search(&self) -> GlobalSearch {
        let search = self.store.global_search();
        GlobalSearch::new(search)
    }

    /// Legacy operations that need to be moved to other crates.
    pub fn legacy(&self) -> Legacy {
        let legacy = self.store.legacy();
        Legacy::new(legacy)
    }

    /// Operate on all namespaces in the cluster.
    pub fn namespace(&self, namespace_id: String) -> Namespace {
        let namespace = self.store.namespace();
        let attrs = self::namespace::NamespaceAttributes {
            ns_id: namespace_id,
        };
        Namespace::new(namespace, attrs)
    }

    /// Operate on all namespaces on the RepliCore instance.
    pub fn namespaces(&self) -> Namespaces {
        let namespaces = self.store.namespaces();
        Namespaces::new(namespaces)
    }

    /// Operate on the node identified by the provided cluster_id and node_id.
    pub fn node(&self, cluster_id: String, node_id: String) -> Node {
        let node = self.store.node();
        let attrs = self::node::NodeAttributes {
            cluster_id,
            node_id,
        };
        Node::new(node, attrs)
    }

    /// Operate on all nodes in the cluster identified by cluster_id.
    pub fn nodes(&self, cluster_id: String) -> Nodes {
        let nodes = self.store.nodes();
        let attrs = self::nodes::NodesAttributes { cluster_id };
        Nodes::new(nodes, attrs)
    }

    /// Operate on the orchestrator action identified by the provided cluster_id and action_id.
    pub fn orchestrator_action<S>(&self, cluster_id: S, action_id: Uuid) -> OrchestratorAction
    where
        S: Into<String>,
    {
        let action = self.store.orchestrator_action();
        let attrs = self::orchestrator_action::OrchestratorActionAttributes {
            action_id,
            cluster_id: cluster_id.into(),
        };
        OrchestratorAction::new(action, attrs)
    }

    /// Operate on all orchestrator actions in the cluster identified by cluster_id.
    pub fn orchestrator_actions(&self, cluster_id: String) -> OrchestratorActions {
        let orchestrator_actions = self.store.orchestrator_actions();
        let attrs = self::orchestrator_actions::OrchestratorActionsAttributes { cluster_id };
        OrchestratorActions::new(orchestrator_actions, attrs)
    }

    /// Persist (insert or update) models to the store.
    pub fn persist(&self) -> Persist {
        let persist = self.store.persist();
        Persist::new(persist)
    }

    /// Operate on the `Platform` identified by the provided namespace and platform IDs.
    pub fn platform<S1, S2>(&self, ns_id: S1, platform_id: S2) -> Platform
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let platform = self.store.platform();
        let attrs = self::platform::PlatformAttributes {
            ns_id: ns_id.into(),
            platform_id: platform_id.into(),
        };
        Platform::new(platform, attrs)
    }

    /// Operate on all `Platform`s in the given namespace.
    pub fn platforms<S1>(&self, ns_id: S1) -> Platforms
    where
        S1: Into<String>,
    {
        let platforms = self.store.platforms();
        let attrs = self::platforms::PlatformsAttributes {
            ns_id: ns_id.into(),
        };
        Platforms::new(platforms, attrs)
    }

    /// Operate on the shard identified by the provided cluster_id, node_id, shard_id.
    pub fn shard(&self, cluster_id: String, node_id: String, shard_id: String) -> Shard {
        let shard = self.store.shard();
        let attrs = self::shard::ShardAttributes {
            cluster_id,
            node_id,
            shard_id,
        };
        Shard::new(shard, attrs)
    }

    /// Operate on all shards in the cluster identified by cluster_id.
    pub fn shards(&self, cluster_id: String) -> Shards {
        let shards = self.store.shards();
        let attrs = self::shards::ShardsAttributes { cluster_id };
        Shards::new(shards, attrs)
    }

    /// Build a synthetic cluster view from individual records.
    pub fn cluster_view<S>(
        &self,
        namespace: String,
        cluster_id: String,
        span: S,
    ) -> Result<ClusterView>
    where
        S: Into<Option<SpanContext>>,
    {
        let full_id = format!("{}.{}", &namespace, &cluster_id);
        let span = span.into();

        // Create the builder.
        let settings = self
            .cluster(namespace.clone(), cluster_id.clone())
            .settings(span.clone())?
            .ok_or_else(|| ErrorKind::RecordNotFound("settings", full_id.clone()))?;
        let discovery = self
            .cluster(namespace.clone(), cluster_id.clone())
            .discovery(span.clone())?
            .ok_or(ErrorKind::RecordNotFound("discovery", full_id))?;
        let mut view = ClusterView::builder(settings, discovery)
            .map_err(AnyWrap::from)
            .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;

        // Add records to the builder.
        for agent in self.agents(cluster_id.clone()).iter(span.clone())? {
            view.agent(agent?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }
        for info in self.agents(cluster_id.clone()).iter_info(span.clone())? {
            view.agent_info(info?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }
        for node in self.nodes(cluster_id.clone()).iter(span.clone())? {
            view.node(node?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }
        for shard in self.shards(cluster_id.clone()).iter(span.clone())? {
            view.shard(shard?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }

        // Add unfinished actions to the builder.
        let actions = self
            .actions(cluster_id.clone())
            .unfinished_summaries(span.clone());
        for summary in actions? {
            view.action(summary?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }

        // Add unfinished orchestrator actions to the builder.
        let actions = self
            .orchestrator_actions(cluster_id.clone())
            .unfinished_summaries(span);
        for summary in actions? {
            view.orchestrator_action(summary?)
                .map_err(AnyWrap::from)
                .with_context(|_| ErrorKind::ViewBuild(namespace.clone(), cluster_id.clone()))?;
        }

        // Build and return the cluster view.
        Ok(view.build())
    }
}
