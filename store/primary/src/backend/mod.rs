use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::Logger;
use uuid::Uuid;

use replicante_externals_mongodb::admin::ValidationResult;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::admin::Version;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::api::orchestrator_action::OrchestratorActionSummary;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::discovery::DiscoverySettings;
use replicante_models_core::cluster::ClusterMeta;
use replicante_models_core::cluster::ClusterSettings;
use replicante_service_healthcheck::HealthChecks;

use crate::store::action::ActionAttributes;
use crate::store::actions::ActionsAttributes;
use crate::store::agent::AgentAttributes;
use crate::store::agents::AgentsAttributes;
use crate::store::cluster::ClusterAttributes;
use crate::store::discovery_settings::DiscoverySettingsAttributes;
use crate::store::node::NodeAttributes;
use crate::store::nodes::NodesAttributes;
use crate::store::orchestrator_action::OrchestratorActionAttributes;
use crate::store::orchestrator_actions::OrchestratorActionsAttributes;
use crate::store::shard::ShardAttributes;
use crate::store::shards::ShardsAttributes;
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

arc_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct AdminImpl,

    /// Definition of top level store administration operations.
    ///
    /// Mainly a way to return interfaces to grouped store operations.
    ///
    /// See `admin::Admin` for descriptions of methods.
    trait AdminInterface,

    interface {
        fn data(&self) -> DataImpl;
        fn validate(&self) -> ValidateImpl;
        fn version(&self) -> Result<Version>;
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
        fn action(&self) -> ActionImpl;
        fn agent(&self) -> AgentImpl;
        fn agents(&self) -> AgentsImpl;
        fn cluster(&self) -> ClusterImpl;
        fn discovery_settings(&self) -> DiscoverySettingsImpl;
        fn global_search(&self) -> GlobalSearchImpl;
        fn legacy(&self) -> LegacyImpl;
        fn node(&self) -> NodeImpl;
        fn nodes(&self) -> NodesImpl;
        fn orchestrator_action(&self) -> OrchestratorActionImpl;
        fn orchestrator_actions(&self) -> OrchestratorActionsImpl;
        fn persist(&self) -> PersistImpl;
        fn shard(&self) -> ShardImpl;
        fn shards(&self) -> ShardsImpl;
    }
}

box_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionImpl,

    /// Definition of supported operations on `Action`s.
    ///
    /// See `store::action::Action` for descriptions of methods.
    trait ActionInterface,

    interface {
        fn get(
            &self,
            attrs: &ActionAttributes,
            span: Option<SpanContext>,
        ) -> Result<Option<Action>>;
    }
}

box_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionsImpl,

    /// Definition of supported operations on `Action`s.
    ///
    /// See `store::actions::Actions` for descriptions of methods.
    trait ActionsInterface,

    interface {
        fn approve(
            &self,
            attrs: &ActionsAttributes,
            action_id: Uuid,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn disapprove(
            &self,
            attrs: &ActionsAttributes,
            action_id: Uuid,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn unfinished_summaries(
            &self,
            attrs: &ActionsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<ActionSyncSummary>>;
    }
}

box_interface! {
    /// Dynamic dispatch agent operations to a backend-specific implementation.
    struct AgentImpl,

    /// Definition of supported operations on `Agent`s and `AgentInfo`s.
    ///
    /// See `store::agent::Agent` for descriptions of methods.
    trait AgentInterface,

    interface {
        fn get(&self, attrs: &AgentAttributes, span: Option<SpanContext>) -> Result<Option<Agent>>;
        fn info(&self, attrs: &AgentAttributes, span: Option<SpanContext>)
            -> Result<Option<AgentInfo>>;
    }
}

box_interface! {
    /// Dynamic dispatch agents operations to a backend-specific implementation.
    struct AgentsImpl,

    /// Definition of supported operations on all agents in a cluster.
    ///
    /// See `store::agents::Agents` for descriptions of methods.
    trait AgentsInterface,

    interface {
        fn iter(
            &self,
            attrs: &AgentsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<Agent>>;
        fn iter_info(
            &self,
            attrs: &AgentsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<AgentInfo>>;
    }
}

box_interface! {
    /// Dynamic dispatch all cluster operations to a backend-specific implementation.
    struct ClusterImpl,

    /// Definition of supported operations on clusters.
    ///
    /// See `store::cluster::Cluster` for descriptions of methods.
    trait ClusterInterface,

    interface {
        fn discovery(
            &self,
            attrs: &ClusterAttributes,
            span: Option<SpanContext>,
        ) -> Result<Option<ClusterDiscovery>>;
        fn settings(
            &self,
            attrs: &ClusterAttributes,
            span: Option<SpanContext>,
        ) -> Result<Option<ClusterSettings>>;
    }
}

box_interface! {
    /// Dynamic dispatch all data admin operations to a backend-specific implementation.
    struct DataImpl,

    /// Definition of supported data admin operations.
    ///
    /// See `admin::data::Data` for descriptions of methods.
    trait DataInterface,

    interface {
        fn actions(&self) -> Result<Cursor<Action>>;
        fn agents(&self) -> Result<Cursor<Agent>>;
        fn agents_info(&self) -> Result<Cursor<AgentInfo>>;
        fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>>;
        fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>>;
        fn nodes(&self) -> Result<Cursor<Node>>;
        fn shards(&self) -> Result<Cursor<Shard>>;
    }
}

box_interface! {
    /// Dynamic dispatch discovery settings operations to a backend-specific implementation.
    struct DiscoverySettingsImpl,

    /// Definition of supported discovery settings operations.
    ///
    /// See `store::discovery_settings::DiscoverySettings` for descriptions of methods.
    trait DiscoverySettingsInterface,

    interface {
        fn delete(
            &self,
            attrs: &DiscoverySettingsAttributes,
            name: &str,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn iter_names(
            &self,
            attrs: &DiscoverySettingsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<String>>;
    }
}

box_interface! {
    /// Dynamic dispatch global search operations to a backend-specific implementation.
    struct GlobalSearchImpl,

    /// Definition of supported global searches.
    ///
    /// See `store::global_search::GlobalSearch` for descriptions of methods.
    trait GlobalSearchInterface,

    interface {
        fn clusters_to_orchestrate(
            &self,
            span: Option<SpanContext>,
        ) -> Result<Cursor<ClusterSettings>>;
        fn discoveries_to_run(
            &self,
            span: Option<SpanContext>,
        ) -> Result<Cursor<DiscoverySettings>>;
    }
}

box_interface! {
    /// Dynamic dispatch legacy operations to a backend-specific implementation.
    struct LegacyImpl,

    /// Definition of legacy operations.
    ///
    /// See `store::legacy::Legacy` for descriptions of methods.
    trait LegacyInterface,

    interface {
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
}

box_interface! {
    /// Dynamic dispatch node operations to a backend-specific implementation.
    struct NodeImpl,

    /// Definition of supported operations on nodes.
    ///
    /// See `store::node::Node` for descriptions of methods.
    trait NodeInterface,

    interface {
        fn get(&self, attrs: &NodeAttributes, span: Option<SpanContext>) -> Result<Option<Node>>;
    }
}

box_interface! {
    /// Dynamic dispatch nodes operations to a backend-specific implementation.
    struct NodesImpl,

    /// Definition of supported operations on all nodes in a cluster.
    ///
    /// See `store::nodes::Nodes` for descriptions of methods.
    trait NodesInterface,

    interface {
        fn iter(&self, attrs: &NodesAttributes, span: Option<SpanContext>) -> Result<Cursor<Node>>;
    }
}

box_interface! {
    /// Dynamic dispatch orchestrator action operations to a backend-specific implementation.
    struct OrchestratorActionImpl,

    /// Definition of supported operations on specific orchestrator actions in a cluster.
    ///
    /// See `store::orchestrator_action::OrchestratorAction` for descriptions of methods.
    trait OrchestratorActionInterface,

    interface {
        fn get(
            &self,
            attrs: &OrchestratorActionAttributes,
            span: Option<SpanContext>,
        ) -> Result<Option<OrchestratorAction>>;
    }
}

box_interface! {
    /// Dynamic dispatch orchestrator actions operations to a backend-specific implementation.
    struct OrchestratorActionsImpl,

    /// Definition of supported operations on all orchestrator actions in a cluster.
    ///
    /// See `store::orchestrator_actions::OrchestratorActions` for descriptions of methods.
    trait OrchestratorActionsInterface,

    interface {
        fn approve(
            &self,
            attrs: &OrchestratorActionsAttributes,
            action_id: Uuid,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn disapprove(
            &self,
            attrs: &OrchestratorActionsAttributes,
            action_id: Uuid,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn iter_summary(
            &self,
            attrs: &OrchestratorActionsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<OrchestratorActionSummary>>;
        fn unfinished_summaries(
            &self,
            attrs: &OrchestratorActionsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<OrchestratorActionSyncSummary>>;
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
        fn cluster_settings(
            &self,
            settings: ClusterSettings,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn discovery_settings(
            &self,
            settings: DiscoverySettings,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn next_cluster_orchestrate(
            &self,
            settings: ClusterSettings,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn next_discovery_run(
            &self,
            settings: DiscoverySettings,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn node(&self, node: Node, span: Option<SpanContext>) -> Result<()>;
        fn orchestrator_action(
            &self,
            action: OrchestratorAction,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn shard(&self, shard: Shard, span: Option<SpanContext>) -> Result<()>;
    }
}

box_interface! {
    /// Dynamic dispatch shard operations to a backend-specific implementation.
    struct ShardImpl,

    /// Definition of supported operations on a shard.
    ///
    /// See `store::shard::Shard` for descriptions of methods.
    trait ShardInterface,

    interface {
        fn get(&self, attrs: &ShardAttributes, span: Option<SpanContext>) -> Result<Option<Shard>>;
    }
}

box_interface! {
    /// Dynamic dispatch shards operations to a backend-specific implementation.
    struct ShardsImpl,

    /// Definition of supported operations on all shards in a cluster.
    ///
    /// See `store::shards::Shards` for descriptions of methods.
    trait ShardsInterface,

    interface {
        fn iter(
            &self,
            attrs: &ShardsAttributes,
            span: Option<SpanContext>,
        ) -> Result<Cursor<Shard>>;
    }
}

box_interface! {
    /// Dynamic dispatch validate operations to a backend-specific implementation.
    struct ValidateImpl,

    /// Definition of supported validation operations.
    ///
    /// See `admin::validate::Validate` for descriptions of methods.
    trait ValidateInterface,

    interface {
        fn removed_entities(&self) -> Result<Vec<ValidationResult>>;
        fn schema(&self) -> Result<Vec<ValidationResult>>;
    }
}
