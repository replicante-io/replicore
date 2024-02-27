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

use crate::config::MongoDBConfig;
use crate::ErrorKind;
use crate::Result;

use super::ActionsImpl;
use super::AdminInterface;
use super::ClusterImpl;
use super::DataImpl;
use super::EventsImpl;
use super::OrchestratorActionsImpl;
use super::PersistImpl;
use super::StoreInterface;
use super::ValidateImpl;

mod actions;
mod cluster;
mod constants;
mod data;
mod document;
mod events;
mod orchestrator_actions;
mod persist;
mod validate;

/// View store admin using MongoDB.
pub struct Admin {
    client: Client,
    db: String,
}

impl Admin {
    pub fn make(config: MongoDBConfig, logger: Logger) -> Result<Admin> {
        info!(logger, "Initialising view store admin for MongoDB");
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

/// View store implementation using MongoDB.
///
/// # Special collection requirements
///
///   * `events`: is a capped or TTL indexed collection.
pub struct Store {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Store {
    /// Create a mongodb-backed view store interface.
    pub fn new<T>(
        config: MongoDBConfig,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Store>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        info!(logger, "Initialising view store backed by MongoDB");
        let db = config.db.clone();
        let client = Client::with_uri_str(&config.common.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.common.uri.clone()))?;
        let tracer = tracer.into();
        let healthcheck = MongoDBHealthCheck::new(client.clone());
        healthchecks.register("store:view", healthcheck);
        Ok(Store { client, db, tracer })
    }
}

impl StoreInterface for Store {
    fn actions(&self, cluster_id: String) -> ActionsImpl {
        let actions = self::actions::Actions::new(
            self.client.clone(),
            self.db.clone(),
            self.tracer.clone(),
            cluster_id,
        );
        ActionsImpl::new(actions)
    }

    fn cluster(&self) -> ClusterImpl {
        let cluster =
            self::cluster::Cluster::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        ClusterImpl::new(cluster)
    }

    fn events(&self) -> EventsImpl {
        let events =
            self::events::Events::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        EventsImpl::new(events)
    }

    fn orchestrator_actions(&self, cluster_id: String) -> OrchestratorActionsImpl {
        let actions = self::orchestrator_actions::OrchestratorActions::new(
            self.client.clone(),
            self.db.clone(),
            self.tracer.clone(),
            cluster_id,
        );
        OrchestratorActionsImpl::new(actions)
    }

    fn persist(&self) -> PersistImpl {
        let persist =
            self::persist::Persist::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        PersistImpl::new(persist)
    }
}
