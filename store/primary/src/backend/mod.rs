use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use prometheus::Registry;
use slog::Logger;

use replicante_models_core::admin::Version;
use replicante_models_core::Agent;
use replicante_models_core::AgentInfo;
use replicante_models_core::ClusterDiscovery;
use replicante_models_core::ClusterMeta;
use replicante_models_core::Event;
use replicante_models_core::Node;
use replicante_models_core::Shard;
use replicante_service_healthcheck::HealthChecks;

use super::admin::ValidationResult;
use super::store::agent::AgentAttribures;
use super::store::agents::AgentsAttribures;
use super::store::agents::AgentsCounts;
use super::store::cluster::ClusterAttribures;
use super::store::legacy::EventsFilters;
use super::store::legacy::EventsOptions;
use super::store::node::NodeAttribures;
use super::store::nodes::NodesAttribures;
use super::store::shard::ShardAttribures;
use super::store::shards::ShardsAttribures;
use super::store::shards::ShardsCounts;
use super::Config;
use super::Cursor;
use super::Result;

mod mongo;

/// Instantiate a new storage backend based on the given configuration.
pub fn backend_factory<T>(
    config: Config,
    logger: Logger,
    healthchecks: &mut HealthChecks,
    tracer: T,
) -> Result<StoreImpl>
where
    T: Into<Option<Arc<Tracer>>>,
{
    let store = match config {
        Config::MongoDB(config) => {
            let store = self::mongo::Store::make(config, logger, healthchecks, tracer)?;
            StoreImpl::new(store)
        }
    };
    Ok(store)
}

/// Instantiate a new storage admin backend based on the given configuration.
pub fn backend_factory_admin(config: Config, logger: Logger) -> Result<AdminImpl> {
    let admin = match config {
        Config::MongoDB(config) => AdminImpl::new(self::mongo::Admin::make(config, logger)?),
    };
    Ok(admin)
}

pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::mongo::register_metrics(logger, registry);
}

/// Definition of top level store administration operations.
///
/// Mainly a way to return interfaces to grouped store operations.
///
/// See `admin::Admin` for descriptions of methods.
pub trait AdminInterface: Send + Sync {
    fn data(&self) -> DataImpl;
    fn validate(&self) -> ValidateImpl;
    fn version(&self) -> Result<Version>;
}

/// Dynamic dispatch all operations to a backend-specific implementation.
#[derive(Clone)]
pub struct AdminImpl(Arc<dyn AdminInterface>);

impl AdminImpl {
    pub fn new<A: AdminInterface + 'static>(admin: A) -> AdminImpl {
        AdminImpl(Arc::new(admin))
    }
}

impl Deref for AdminImpl {
    type Target = dyn AdminInterface + 'static;
    fn deref(&self) -> &(dyn AdminInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on `Agent`s and `AgentInfo`s.
///
/// See `store::agent::Agent` for descriptions of methods.
pub trait AgentInterface: Send + Sync {
    fn get(&self, attrs: &AgentAttribures, span: Option<SpanContext>) -> Result<Option<Agent>>;
    fn info(&self, attrs: &AgentAttribures, span: Option<SpanContext>)
        -> Result<Option<AgentInfo>>;
}

/// Dynamic dispatch agent operations to a backend-specific implementation.
#[derive(Clone)]
pub struct AgentImpl(Arc<dyn AgentInterface>);

impl AgentImpl {
    pub fn new<A: AgentInterface + 'static>(agent: A) -> AgentImpl {
        AgentImpl(Arc::new(agent))
    }
}

impl Deref for AgentImpl {
    type Target = dyn AgentInterface + 'static;
    fn deref(&self) -> &(dyn AgentInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on all agents in a cluster.
///
/// See `store::agents::Agents` for descriptions of methods.
pub trait AgentsInterface: Send + Sync {
    fn counts(&self, attrs: &AgentsAttribures, span: Option<SpanContext>) -> Result<AgentsCounts>;
    fn iter(&self, attrs: &AgentsAttribures, span: Option<SpanContext>) -> Result<Cursor<Agent>>;
    fn iter_info(
        &self,
        attrs: &AgentsAttribures,
        span: Option<SpanContext>,
    ) -> Result<Cursor<AgentInfo>>;
}

/// Dynamic dispatch agents operations to a backend-specific implementation.
#[derive(Clone)]
pub struct AgentsImpl(Arc<dyn AgentsInterface>);

impl AgentsImpl {
    pub fn new<A: AgentsInterface + 'static>(agents: A) -> AgentsImpl {
        AgentsImpl(Arc::new(agents))
    }
}

impl Deref for AgentsImpl {
    type Target = dyn AgentsInterface + 'static;
    fn deref(&self) -> &(dyn AgentsInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on clusters.
///
/// See `store::cluster::Cluster` for descriptions of methods.
pub trait ClusterInterface: Send + Sync {
    fn discovery(
        &self,
        attrs: &ClusterAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<ClusterDiscovery>>;
    fn mark_stale(&self, attrs: &ClusterAttribures, span: Option<SpanContext>) -> Result<()>;
}

/// Dynamic dispatch all cluster operations to a backend-specific implementation.
#[derive(Clone)]
pub struct ClusterImpl(Arc<dyn ClusterInterface>);

impl ClusterImpl {
    pub fn new<C: ClusterInterface + 'static>(cluster: C) -> ClusterImpl {
        ClusterImpl(Arc::new(cluster))
    }
}

impl Deref for ClusterImpl {
    type Target = dyn ClusterInterface + 'static;
    fn deref(&self) -> &(dyn ClusterInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported data admin operations.
///
/// See `admin::data::Data` for descriptions of methods.
pub trait DataInterface: Send + Sync {
    fn agents(&self) -> Result<Cursor<Agent>>;
    fn agents_info(&self) -> Result<Cursor<AgentInfo>>;
    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>>;
    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>>;
    fn events(&self) -> Result<Cursor<Event>>;
    fn nodes(&self) -> Result<Cursor<Node>>;
    fn shards(&self) -> Result<Cursor<Shard>>;
}

/// Dynamic dispatch all data admin operations to a backend-specific implementation.
#[derive(Clone)]
pub struct DataImpl(Arc<dyn DataInterface>);

impl DataImpl {
    pub fn new<D: DataInterface + 'static>(data: D) -> DataImpl {
        DataImpl(Arc::new(data))
    }
}

impl Deref for DataImpl {
    type Target = dyn DataInterface + 'static;
    fn deref(&self) -> &(dyn DataInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of legacy operations.
///
/// See `store::legacy::Legacy` for descriptions of methods.
pub trait LegacyInterface: Send + Sync {
    fn cluster_meta(
        &self,
        cluster_id: String,
        span: Option<SpanContext>,
    ) -> Result<Option<ClusterMeta>>;
    fn events(
        &self,
        filters: EventsFilters,
        options: EventsOptions,
        span: Option<SpanContext>,
    ) -> Result<Cursor<Event>>;
    fn find_clusters(
        &self,
        search: String,
        limit: u8,
        span: Option<SpanContext>,
    ) -> Result<Cursor<ClusterMeta>>;
    fn persist_cluster_meta(&self, meta: ClusterMeta, span: Option<SpanContext>) -> Result<()>;
    fn persist_event(&self, event: Event, span: Option<SpanContext>) -> Result<()>;
    fn top_clusters(&self, span: Option<SpanContext>) -> Result<Cursor<ClusterMeta>>;
}

/// Dynamic dispatch legacy operations to a backend-specific implementation.
#[derive(Clone)]
pub struct LegacyImpl(Arc<dyn LegacyInterface>);

impl LegacyImpl {
    pub fn new<L: LegacyInterface + 'static>(legacy: L) -> LegacyImpl {
        LegacyImpl(Arc::new(legacy))
    }
}

impl Deref for LegacyImpl {
    type Target = dyn LegacyInterface + 'static;
    fn deref(&self) -> &(dyn LegacyInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on nodes.
///
/// See `store::node::Node` for descriptions of methods.
pub trait NodeInterface: Send + Sync {
    fn get(&self, attrs: &NodeAttribures, span: Option<SpanContext>) -> Result<Option<Node>>;
}

/// Dynamic dispatch node operations to a backend-specific implementation.
#[derive(Clone)]
pub struct NodeImpl(Arc<dyn NodeInterface>);

impl NodeImpl {
    pub fn new<N: NodeInterface + 'static>(node: N) -> NodeImpl {
        NodeImpl(Arc::new(node))
    }
}

impl Deref for NodeImpl {
    type Target = dyn NodeInterface + 'static;
    fn deref(&self) -> &(dyn NodeInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on all nodes in a cluster.
///
/// See `store::nodes::Nodes` for descriptions of methods.
pub trait NodesInterface: Send + Sync {
    fn iter(&self, attrs: &NodesAttribures, span: Option<SpanContext>) -> Result<Cursor<Node>>;
    fn kinds(&self, attrs: &NodesAttribures, span: Option<SpanContext>) -> Result<HashSet<String>>;
}

/// Dynamic dispatch nodes operations to a backend-specific implementation.
#[derive(Clone)]
pub struct NodesImpl(Arc<dyn NodesInterface>);

impl NodesImpl {
    pub fn new<N: NodesInterface + 'static>(nodes: N) -> NodesImpl {
        NodesImpl(Arc::new(nodes))
    }
}

impl Deref for NodesImpl {
    type Target = dyn NodesInterface + 'static;
    fn deref(&self) -> &(dyn NodesInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of model persist operations.
///
/// See `store::persist::Persist` for descriptions of methods.
pub trait PersistInterface: Send + Sync {
    fn agent(&self, agent: Agent, span: Option<SpanContext>) -> Result<()>;
    fn agent_info(&self, agent: AgentInfo, span: Option<SpanContext>) -> Result<()>;
    fn cluster_discovery(
        &self,
        discovery: ClusterDiscovery,
        span: Option<SpanContext>,
    ) -> Result<()>;
    fn node(&self, node: Node, span: Option<SpanContext>) -> Result<()>;
    fn shard(&self, shard: Shard, span: Option<SpanContext>) -> Result<()>;
}

/// Dynamic dispatch persist operations to a backend-specific implementation.
#[derive(Clone)]
pub struct PersistImpl(Arc<dyn PersistInterface>);

impl PersistImpl {
    pub fn new<P: PersistInterface + 'static>(persist: P) -> PersistImpl {
        PersistImpl(Arc::new(persist))
    }
}

impl Deref for PersistImpl {
    type Target = dyn PersistInterface + 'static;
    fn deref(&self) -> &(dyn PersistInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of top level store operations.
///
/// Mainly a way to return interfaces to grouped store operations.
///
/// See `store::Store` for descriptions of methods.
pub trait StoreInterface: Send + Sync {
    fn agent(&self) -> AgentImpl;
    fn agents(&self) -> AgentsImpl;
    fn cluster(&self) -> ClusterImpl;
    fn legacy(&self) -> LegacyImpl;
    fn node(&self) -> NodeImpl;
    fn nodes(&self) -> NodesImpl;
    fn persist(&self) -> PersistImpl;
    fn shard(&self) -> ShardImpl;
    fn shards(&self) -> ShardsImpl;
}

/// Dynamic dispatch all operations to a backend-specific implementation.
#[derive(Clone)]
pub struct StoreImpl(Arc<dyn StoreInterface>);

impl StoreImpl {
    pub fn new<S: StoreInterface + 'static>(store: S) -> StoreImpl {
        StoreImpl(Arc::new(store))
    }
}

impl Deref for StoreImpl {
    type Target = dyn StoreInterface + 'static;
    fn deref(&self) -> &(dyn StoreInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on a shard.
///
/// See `store::shard::Shard` for descriptions of methods.
pub trait ShardInterface: Send + Sync {
    fn get(&self, attrs: &ShardAttribures, span: Option<SpanContext>) -> Result<Option<Shard>>;
}

/// Dynamic dispatch shard operations to a backend-specific implementation.
#[derive(Clone)]
pub struct ShardImpl(Arc<dyn ShardInterface>);

impl ShardImpl {
    pub fn new<S: ShardInterface + 'static>(shard: S) -> ShardImpl {
        ShardImpl(Arc::new(shard))
    }
}

impl Deref for ShardImpl {
    type Target = dyn ShardInterface + 'static;
    fn deref(&self) -> &(dyn ShardInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported operations on all shards in a cluster.
///
/// See `store::shards::Shards` for descriptions of methods.
pub trait ShardsInterface: Send + Sync {
    fn counts(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<ShardsCounts>;
    fn iter(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<Cursor<Shard>>;
}

#[derive(Clone)]
pub struct ShardsImpl(Arc<dyn ShardsInterface>);

impl ShardsImpl {
    pub fn new<S: ShardsInterface + 'static>(shards: S) -> ShardsImpl {
        ShardsImpl(Arc::new(shards))
    }
}

impl Deref for ShardsImpl {
    type Target = dyn ShardsInterface + 'static;
    fn deref(&self) -> &(dyn ShardsInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported validation operations.
///
/// See `admin::validate::Validate` for descriptions of methods.
pub trait ValidateInterface: Send + Sync {
    fn indexes(&self) -> Result<Vec<ValidationResult>>;
    fn removed_entities(&self) -> Result<Vec<ValidationResult>>;
    fn schema(&self) -> Result<Vec<ValidationResult>>;
}

/// Dynamic dispatch validate operations to a backend-specific implementation.
#[derive(Clone)]
pub struct ValidateImpl(Arc<dyn ValidateInterface>);

impl ValidateImpl {
    pub fn new<V: ValidateInterface + 'static>(validate: V) -> ValidateImpl {
        ValidateImpl(Arc::new(validate))
    }
}

impl Deref for ValidateImpl {
    type Target = dyn ValidateInterface + 'static;
    fn deref(&self) -> &(dyn ValidateInterface + 'static) {
        self.0.deref()
    }
}
