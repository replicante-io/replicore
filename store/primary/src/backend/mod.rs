use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::Logger;
use uuid::Uuid;

use replicante_externals_mongodb::admin::ValidationResult;
use replicante_models_core::actions::Action;
use replicante_models_core::admin::Version;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::ClusterDiscovery;
use replicante_models_core::cluster::ClusterMeta;
use replicante_service_healthcheck::HealthChecks;

use crate::store::actions::ActionSyncState;
use crate::store::actions::ActionsAttributes;
use crate::store::agent::AgentAttribures;
use crate::store::agents::AgentsAttribures;
use crate::store::agents::AgentsCounts;
use crate::store::cluster::ClusterAttribures;
use crate::store::node::NodeAttribures;
use crate::store::nodes::NodesAttribures;
use crate::store::shard::ShardAttribures;
use crate::store::shards::ShardsAttribures;
use crate::store::shards::ShardsCounts;
use crate::Config;
use crate::Cursor;
use crate::Result;

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

// Macro definition to generate an interface trait with a wrapping wrapper
// for dynamic dispatch to Send + Sync + 'static implementations.
macro_rules! arc_interface {
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name: Send + Sync $trait_def

        $(#[$struct_meta])*
        #[derive(Clone)]
        pub struct $struct_name(Arc<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Arc::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }
    }
}
macro_rules! box_interface {
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name $trait_def

        $(#[$struct_meta])*
        pub struct $struct_name(Box<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Box::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut (dyn $trait_name + 'static) {
                self.0.deref_mut()
            }
        }
    };
}

box_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionsImpl,

    /// Definition of supported operations on `Action`s.
    ///
    /// See `store::actions::Actions` for descriptions of methods.
    trait ActionsInterface,

    interface {
        fn mark_lost(
            &self,
            attrs: &ActionsAttributes,
            node_id: String,
            refresh_id: i64,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn state_for_sync(
            &self,
            attrs: &ActionsAttributes,
            node_id: String,
            action_ids: &[Uuid],
            span: Option<SpanContext>,
        ) -> Result<HashMap<Uuid, ActionSyncState>>;
    }
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
    fn find_clusters(
        &self,
        search: String,
        limit: u8,
        span: Option<SpanContext>,
    ) -> Result<Cursor<ClusterMeta>>;
    fn persist_cluster_meta(&self, meta: ClusterMeta, span: Option<SpanContext>) -> Result<()>;
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

box_interface! {
    /// Dynamic dispatch persist operations to a backend-specific implementation.
    struct PersistImpl,

    /// Definition of model persist operations.
    ///
    /// See `store::persist::Persist` for descriptions of methods.
    trait PersistInterface,

    interface {
        fn action(&self, action: Action, span: Option<SpanContext>) -> Result<()>;
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
}

arc_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct StoreImpl,

    /// Definition of top level store operations.
    ///
    /// Mainly a way to return interfaces to grouped store operations.
    ///
    /// See `store::Store` for descriptions of methods.
    trait StoreInterface,

    interface {
        fn actions(&self) -> ActionsImpl;
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
