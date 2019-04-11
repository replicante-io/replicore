use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::Config;
use super::Cursor;
use super::Result;
use super::backend::mongo::MongoStore;


/// Filters to apply when iterating over events.
pub struct EventsFilters {
    /// Only return cluster-related events if the cluster ID matches.
    ///
    /// Non-cluster events will still be returned.
    pub cluster_id: Option<String>,

    /// Only return events with a matching event code.
    pub event: Option<String>,

    /// Exclude snapshot events from the result (on by default).
    pub exclude_snapshots: bool,

    /// Exclude events that do not relate to a cluster (off by default).
    pub exclude_system_events: bool,

    /// Scan events starting from the given UTC date and time instead of from the oldest event.
    pub start_from: Option<DateTime<Utc>>,

    /// Scan events up to the given UTC date and time instead of up to the newest event.
    pub stop_at: Option<DateTime<Utc>>,
}

impl EventsFilters {
    /// Return all events, don't skip any.
    pub fn all() -> EventsFilters {
        Self::default()
    }
}

impl Default for EventsFilters {
    fn default() -> EventsFilters {
        EventsFilters {
            cluster_id: None,
            event: None,
            exclude_snapshots: true,
            exclude_system_events: false,
            start_from: None,
            stop_at: None,
        }
    }
}


/// Options to apply when iterating over events.
pub struct EventsOptions {
    /// Max number of events to return.
    pub limit: Option<i64>,

    /// By default events are returned old to new, set to true to reverse the order.
    pub reverse: bool,
}

impl Default for EventsOptions {
    fn default() -> EventsOptions {
        EventsOptions {
            limit: None,
            reverse: false,
        }
    }
}


/// Private interface to the persistence storage layer.
///
/// Allows multiple possible datastores to be used as well as mocks for testing.
pub trait InnerStore: Send + Sync {
    /// See `Store::agent` for details.
    fn agent(&self, cluster_id: String, host: String) -> Result<Option<Agent>>;

    /// See `Store::agent_info` for details.
    fn agent_info(&self, cluster_id: String, host: String) -> Result<Option<AgentInfo>>;

    /// See `Store::cluster_agents` for details.
    fn cluster_agents(&self, cluster_id: String) -> Result<Cursor<Agent>>;

    /// See `Store::cluster_agents_info` for details.
    fn cluster_agents_info(&self, cluster_id: String) -> Result<Cursor<AgentInfo>>;

    /// See `Store::cluster_discovery` for details.
    fn cluster_discovery(&self, cluster_id: String) -> Result<Option<ClusterDiscovery>>;

    /// See `Store::cluster_meta` for details.
    fn cluster_meta(&self, cluster_id: String) -> Result<Option<ClusterMeta>>;

    /// See `Store::cluster_nodes` for details.
    fn cluster_nodes(&self, cluster_id: String) -> Result<Cursor<Node>>;

    /// See `Store::cluster_shards` for details.
    fn cluster_shards(&self, cluster_id: String) -> Result<Cursor<Shard>>;

    /// See `Store::events` for details.
    fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<Cursor<Event>>;

    /// See `Store::find_clusters` for details.
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>>;

    /// See `Store::node` for details.
    fn node(&self, cluster_id: String, name: String) -> Result<Option<Node>>;

    /// See `Some::persist_agent` for details.
    fn persist_agent(&self, agent: Agent) -> Result<()>;

    /// See `Some::persist_agent_info` for details.
    fn persist_agent_info(&self, agent: AgentInfo) -> Result<()>;

    /// See `Store::persist_cluster_meta` for details.
    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()>;

    /// See `Store::persist_event` for details.
    fn persist_event(&self, event: Event) -> Result<()>;

    /// See `Store::persist_discovery` for details.
    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()>;

    /// See `Store::persist_node` for details.
    fn persist_node(&self, node: Node) -> Result<()>;

    /// See `Store::persist_shard` for details.
    fn persist_shard(&self, shard: Shard) -> Result<()>;

    /// See `Store::shard` for details.
    fn shard(&self, cluster_id: String, node: String, id: String) -> Result<Option<Shard>>;

    /// See `Store::top_clusters` for details.
    fn top_clusters(&self) -> Result<Vec<ClusterMeta>>;
}


/// Public interface to the persistent storage layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
///
/// # Overview
/// Different data types (models) are owned by a component and different data units
/// (model instances) are owned by a running replicante component instance.
///
/// Owned means that only one component's running instance can generate or update the data.
/// This avoids most issues with concurrent data updates (it will never be possible to prevent
/// datastores from changing while data is collected from agents).
///
/// Ownership of data also simplifies operations that require a "full view" of the cluster
/// because one conponent's instance will be uniquely responsible for creating this "full view".
///
/// # Data flow
///
///   1. The discovery component (only one active in the entire cluster):
///     1. Discovers clusters with all their nodes.
///     2. Detects new clusters and nodes as well as nodes leaving the cluster.
///   2. The datafetch components (as many as desired, at least one):
///     1. Take exclusive ownership of each cluster.
///     2. Periodically fetch the state of each agent.
///     3. Build cluster metadata documents (incrementally, while iterating over agents).
///     4. Update agents, nodes, and shards models with the new data.
///     5. Compare known state with the newly fetched state to generate events.
#[derive(Clone)]
pub struct Store(Arc<InnerStore>);

impl Store {
    /// Instantiate a new storage interface.
    pub fn new(config: Config, logger: Logger) -> Result<Store> {
        let store = match config {
            Config::MongoDB(config) => Arc::new(MongoStore::new(config, logger)?),
        };
        Ok(Store(store))
    }

    /// Attemps to register all metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        super::backend::mongo::register_metrics(logger, registry);
    }

    /// Fetch agent status information.
    pub fn agent<S1, S2>(&self, cluster_id: S1, host: S2) -> Result<Option<Agent>>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.0.agent(cluster_id.into(), host.into())
    }

    /// Fetch agent information.
    pub fn agent_info<S1, S2>(&self, cluster_id: S1, host: S2) -> Result<Option<AgentInfo>>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.0.agent_info(cluster_id.into(), host.into())
    }

    /// Fetch the status of all agents in a cluster.
    pub fn cluster_agents<S>(&self, cluster_id: S) -> Result<Cursor<Agent>>
    where
        S: Into<String>,
    {
        self.0.cluster_agents(cluster_id.into())
    }

    /// Fetch information about all agents in a cluster.
    pub fn cluster_agents_info<S>(&self, cluster_id: S) -> Result<Cursor<AgentInfo>>
    where
        S: Into<String>,
    {
        self.0.cluster_agents_info(cluster_id.into())
    }

    /// Fetch discovery information about a cluster.
    pub fn cluster_discovery<S>(&self, cluster_id: S) -> Result<Option<ClusterDiscovery>>
    where
        S: Into<String>,
    {
        self.0.cluster_discovery(cluster_id.into())
    }

    /// Fetch metadata about a cluster.
    pub fn cluster_meta<S>(&self, cluster_id: S) -> Result<Option<ClusterMeta>>
    where
        S: Into<String>,
    {
        self.0.cluster_meta(cluster_id.into())
    }

    /// Fetch information about all nodes in a cluster.
    pub fn cluster_nodes<S>(&self, cluster_id: S) -> Result<Cursor<Node>>
    where
        S: Into<String>,
    {
        self.0.cluster_nodes(cluster_id.into())
    }

    /// Fetch information about all shards in a cluster.
    pub fn cluster_shards<S>(&self, cluster_id: S) -> Result<Cursor<Shard>>
    where
        S: Into<String>,
    {
        self.0.cluster_shards(cluster_id.into())
    }

    /// Return an iterator over events in the store.
    ///
    /// Pass `filters` to tune the events that will be returned and `options` to
    /// control result behaviour like limit of items or order (old to new/new to old).
    pub fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<Cursor<Event>> {
        self.0.events(filters, options)
    }

    /// Search for a list of clusters with names matching the search term.
    ///
    /// A limited number of cluster is returned to avoid abuse.
    /// To find more clusters refine the search (paging is not supported).
    pub fn find_clusters<S>(&self, search: S, limit: u8) -> Result<Vec<ClusterMeta>>
    where
        S: Into<String>,
    {
        self.0.find_clusters(search.into(), limit)
    }

    /// Fetch information about a node.
    pub fn node<S1, S2>(&self, cluster_id: S1, name: S2) -> Result<Option<Node>>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.0.node(cluster_id.into(), name.into())
    }

    /// Persist the status of an agent.
    ///
    /// If the agent is known it will be updated and the old model is returned.
    /// Tf the agent is new it will be created and `None` will be returned.
    ///
    /// Agents are uniquely identified by their cluster and host.
    pub fn persist_agent(&self, agent: Agent) -> Result<()> {
        self.0.persist_agent(agent)
    }

    /// Persist information about an agent.
    ///
    /// If the agent is known it will be updated and the old model is returned.
    /// Tf the agent is new it will be created and `None` will be returned.
    ///
    /// Agents are uniquely identified by their cluster and host.
    pub fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        self.0.persist_agent_info(agent)
    }

    /// Persist aggregated information about a cluster.
    ///
    /// Cluster metadata is generated by the agent status fetch process.
    ///
    /// If the cluster is known it will be updated and the old model is returned.
    /// Tf the cluster is new it will be created and `None` will be returned.
    ///
    /// Clusters are uniquely identified by their name.
    pub fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        self.0.persist_cluster_meta(meta)
    }

    /// Persist information about a discovered cluster.
    ///
    /// Clusters are uniquely identified by their name.
    pub fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        self.0.persist_discovery(cluster)
    }

    /// Persist a new event in the system.
    pub fn persist_event(&self, event: Event) -> Result<()> {
        self.0.persist_event(event)
    }


    /// Persist information about a node.
    ///
    /// If the node is known it will be updated and the old model is returned.
    /// Tf the node is new it will be created and `None` will be returned.
    ///
    /// Nodes are uniquely identified by `(cluster, name)`.
    pub fn persist_node(&self, node: Node) -> Result<()> {
        self.0.persist_node(node)
    }

    /// Persist information about a shard.
    ///
    /// If the shard is known it will be updated and the old model is returned.
    /// Tf the shard is new it will be created and `None` will be returned.
    ///
    /// Shards are uniquely identified by `(cluster, node, id)`.
    pub fn persist_shard(&self, shard: Shard) -> Result<()> {
        self.0.persist_shard(shard)
    }

    /// Fetch information about a shard.
    pub fn shard<S1, S2, S3>(&self, cluster_id: S1, node: S2, id: S3) -> Result<Option<Shard>>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        self.0.shard(cluster_id.into(), node.into(), id.into())
    }

    /// Fetch overvew details of the top clusters.
    ///
    /// Clusters are sorted by number of nodes in the cluster.
    pub fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        self.0.top_clusters()
    }

    /// Instantiate a `Store` that wraps the given `MockStore`.
    // Cargo builds dependencies in debug mode instead of test mode.
    // That means that `cfg(test)` cannot be used if the mock is used outside the crate.
    #[cfg(debug_assertions)]
    pub fn mock(inner: Arc<super::mock::MockStore>) -> Store {
        Store(inner)
    }
}
