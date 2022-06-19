use std::sync::Arc;

use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::Tracer;
use slog::info;
use slog::Logger;

use replicante_externals_mongodb::version as detect_version;
use replicante_externals_mongodb::MongoDBHealthCheck;
use replicante_models_core::admin::Version;
use replicante_service_healthcheck::HealthChecks;

use super::ActionImpl;
use super::ActionsImpl;
use super::AdminInterface;
use super::AgentImpl;
use super::AgentsImpl;
use super::ClusterImpl;
use super::DataImpl;
use super::DiscoverySettingsImpl;
use super::GlobalSearchImpl;
use super::LegacyImpl;
use super::NodeImpl;
use super::NodesImpl;
use super::OrchestratorActionsImpl;
use super::PersistImpl;
use super::ShardImpl;
use super::ShardsImpl;
use super::StoreInterface;
use super::ValidateImpl;
use crate::config::MongoDBConfig;
use crate::ErrorKind;
use crate::Result;

mod action;
mod actions;
mod agent;
mod agents;
mod cluster;
mod constants;
mod data;
mod discovery_settings;
mod document;
mod global_search;
mod legacy;
mod node;
mod nodes;
mod orchestrator_actions;
mod persist;
mod shard;
mod shards;
mod validate;

/// Primary store admin using MongoDB.
pub struct Admin {
    client: Client,
    db: String,
}

impl Admin {
    pub fn make(config: MongoDBConfig, logger: Logger) -> Result<Admin> {
        info!(logger, "Initialising primary store admin for MongoDB");
        let db = config.db.clone();
        let client = Client::with_uri_str(&config.common.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.common.uri.clone()))?;
        Ok(Admin { client, db })
    }
}

impl AdminInterface for Admin {
    fn data(&self) -> DataImpl {
        let data = self::data::Data::new(self.client.clone(), self.db.clone());
        DataImpl::new(data)
    }

    fn validate(&self) -> ValidateImpl {
        let validate = self::validate::Validate::new(self.client.clone(), self.db.clone());
        ValidateImpl::new(validate)
    }

    fn version(&self) -> Result<Version> {
        let version =
            detect_version(&self.client, &self.db).with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(version)
    }
}

/// Primary store implementation using MongoDB.
///
/// # Special collection requirements
///
///   * `events`: is a capped or TTL indexed collection.
///
/// # Expected indexes
///
///   * Index on `clusters_meta`: `(shards: -1, nodes: -1, cluster_id: 1)`
///   * Unique index on `agents`: `(cluster_id: 1, host: 1)`
///   * Unique index on `agents_info`: `(cluster_id: 1, host: 1)`
///   * Unique index on `clusters_meta`: `cluster_id: 1`
///   * Unique index on `discoveries`: `cluster_id: 1`
///   * Unique index on `nodes`: `(cluster_id: 1, node_id: 1)`
///   * Unique index on `shards`: `(cluster_id: 1, shard_id: 1, node_id: 1)`
pub struct Store {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Store {
    /// Create a mongodb-backed primary store interface.
    pub fn make<T>(
        config: MongoDBConfig,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Store>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        info!(logger, "Initialising primary store backed by MongoDB");
        let db = config.db.clone();
        let client = Client::with_uri_str(&config.common.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.common.uri.clone()))?;
        let tracer = tracer.into();
        let healthcheck = MongoDBHealthCheck::new(client.clone());
        healthchecks.register("store:primary", healthcheck);
        Ok(Store { client, db, tracer })
    }
}

impl StoreInterface for Store {
    fn action(&self) -> ActionImpl {
        let action =
            self::action::Action::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ActionImpl::new(action)
    }

    fn actions(&self) -> ActionsImpl {
        let actions =
            self::actions::Actions::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ActionsImpl::new(actions)
    }

    fn agent(&self) -> AgentImpl {
        let agent =
            self::agent::Agent::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        AgentImpl::new(agent)
    }

    fn agents(&self) -> AgentsImpl {
        let agents =
            self::agents::Agents::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        AgentsImpl::new(agents)
    }

    fn cluster(&self) -> ClusterImpl {
        let cluster =
            self::cluster::Cluster::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ClusterImpl::new(cluster)
    }

    fn discovery_settings(&self) -> DiscoverySettingsImpl {
        let discovery_settings = self::discovery_settings::DiscoverySettings::new(
            self.client.clone(),
            self.db.clone(),
            self.tracer.clone(),
        );
        DiscoverySettingsImpl::new(discovery_settings)
    }

    fn global_search(&self) -> GlobalSearchImpl {
        let search = self::global_search::GlobalSearch::new(
            self.client.clone(),
            self.db.clone(),
            self.tracer.clone(),
        );
        GlobalSearchImpl::new(search)
    }

    fn legacy(&self) -> LegacyImpl {
        let legacy =
            self::legacy::Legacy::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        LegacyImpl::new(legacy)
    }

    fn node(&self) -> NodeImpl {
        let node = self::node::Node::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        NodeImpl::new(node)
    }

    fn nodes(&self) -> NodesImpl {
        let nodes =
            self::nodes::Nodes::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        NodesImpl::new(nodes)
    }

    fn orchestrator_actions(&self) -> OrchestratorActionsImpl {
        let actions = self::orchestrator_actions::OrchestratorActions::new(
            self.client.clone(),
            self.db.clone(),
            self.tracer.clone(),
        );
        OrchestratorActionsImpl::new(actions)
    }

    fn persist(&self) -> PersistImpl {
        let persist =
            self::persist::Persist::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        PersistImpl::new(persist)
    }

    fn shard(&self) -> ShardImpl {
        let shard =
            self::shard::Shard::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ShardImpl::new(shard)
    }

    fn shards(&self) -> ShardsImpl {
        let shards =
            self::shards::Shards::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ShardsImpl::new(shards)
    }
}
