use std::sync::Arc;

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
use super::Result;
use super::backend::mongo::MongoStore;


/// Iterator over events returned by `Store::events`.
pub struct EventsIter(Box<Iterator<Item=Result<Event>>>);

impl EventsIter {
    pub fn new<I>(iter: I) -> EventsIter
        where I: Iterator<Item=Result<Event>> + 'static
    {
        EventsIter(Box::new(iter))
    }
}

impl Iterator for EventsIter {
    type Item = Result<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}


/// Filters to apply when iterating over events.
pub struct EventsFilters {}

impl EventsFilters {
    /// Return all events, don't skip any.
    pub fn all() -> EventsFilters {
        Self::default()
    }
}

impl Default for EventsFilters {
    fn default() -> EventsFilters {
        EventsFilters { }
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
    fn agent(&self, cluster: String, host: String) -> Result<Option<Agent>>;

    /// See `Store::agent_info` for details.
    fn agent_info(&self, cluster: String, host: String) -> Result<Option<AgentInfo>>;

    /// See `Store::cluster_discovery` for details.
    fn cluster_discovery(&self, cluster: String) -> Result<Option<ClusterDiscovery>>;

    /// See `Store::cluster_meta` for details.
    fn cluster_meta(&self, cluster: String) -> Result<Option<ClusterMeta>>;

    /// See `Store::events` for details.
    fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<EventsIter>;

    /// See `Store::find_clusters` for details.
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>>;

    /// See `Store::node` for details.
    fn node(&self, cluster: String, name: String) -> Result<Option<Node>>;

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
    fn shard(&self, cluster: String, node: String, id: String) -> Result<Option<Shard>>;

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
    pub fn new(config: Config, logger: Logger, registry: &Registry) -> Result<Store> {
        let store = match config {
            Config::MongoDB(config) => Arc::new(MongoStore::new(config, logger, registry)?),
        };
        Ok(Store(store))
    }

    /// Fetch agent status information.
    pub fn agent<S1, S2>(&self, cluster: S1, host: S2) -> Result<Option<Agent>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        self.0.agent(cluster.into(), host.into())
    }

    /// Fetch agent information.
    pub fn agent_info<S1, S2>(&self, cluster: S1, host: S2) -> Result<Option<AgentInfo>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        self.0.agent_info(cluster.into(), host.into())
    }

    /// Fetch discovery information about a cluster.
    pub fn cluster_discovery<S>(&self, cluster: S) -> Result<Option<ClusterDiscovery>>
        where S: Into<String>,
    {
        self.0.cluster_discovery(cluster.into())
    }

    /// Fetch metadata about a cluster.
    pub fn cluster_meta<S>(&self, cluster: S) -> Result<Option<ClusterMeta>>
        where S: Into<String>,
    {
        self.0.cluster_meta(cluster.into())
    }

    /// Return an iterator over events in the store.
    ///
    /// Pass `filters` to tune the events that will be returned and `options` to
    /// control result behavior like limit of items or order (old to new/new to old).
    pub fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<EventsIter> {
        self.0.events(filters, options)
    }

    /// Search for a list of clusters with names matching the search term.
    ///
    /// A limited number of cluster is returned to avoid abuse.
    /// To find more clusters refine the search (paging is not supported).
    pub fn find_clusters<S>(&self, search: S, limit: u8) -> Result<Vec<ClusterMeta>>
        where S: Into<String>,
    {
        self.0.find_clusters(search.into(), limit)
    }

    /// Fetch information about a node.
    pub fn node<S1, S2>(&self, cluster: S1, name: S2) -> Result<Option<Node>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        self.0.node(cluster.into(), name.into())
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
    pub fn shard<S1, S2, S3>(&self, cluster: S1, node: S2, id: S3) -> Result<Option<Shard>>
        where S1: Into<String>,
              S2: Into<String>,
              S3: Into<String>,
    {
        self.0.shard(cluster.into(), node.into(), id.into())
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
