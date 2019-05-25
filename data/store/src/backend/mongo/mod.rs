use std::sync::Arc;

use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::Tracer;
use slog::Logger;

use replicante_data_models::admin::Version;

use super::super::config::MongoDBConfig;
use super::super::ErrorKind;
use super::super::Result;

use super::AdminInterface;
use super::AgentImpl;
use super::AgentsImpl;
use super::ClusterImpl;
use super::DataImpl;
use super::LegacyImpl;
use super::NodeImpl;
use super::NodesImpl;
use super::PersistImpl;
use super::ShardImpl;
use super::ShardsImpl;
use super::StoreInterface;
use super::ValidateImpl;

mod agent;
mod agents;
mod cluster;
mod common;
mod constants;
mod data;
mod document;
mod legacy;
mod metrics;
mod node;
mod nodes;
mod persist;
mod shard;
mod shards;
mod validate;

pub use self::metrics::register_metrics;

/// Primary store admin using MongoDB.
pub struct Admin {
    client: Client,
    db: String,
    logger: Logger,
}

impl Admin {
    pub fn make(config: MongoDBConfig, logger: Logger) -> Result<Admin> {
        info!(logger, "Instantiating primary store admin for MongoDB");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.uri.clone()))?;
        Ok(Admin { client, db, logger })
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
        let db = self.client.db(&self.db);
        let version = db
            .version()
            .with_context(|_| ErrorKind::MongoDBOperation("version"))?;
        // The mongodb crate uses semver ^0.8.0 while replicante uses latest.
        // "Convert" the version object across crate versions.
        let version: ::semver::Version = match version.to_string().parse() {
            Ok(version) => version,
            Err(_) => {
                warn!(self.logger, "Failed to convert response to semver"; "version" => %version);
                ::semver::Version::new(version.major, version.minor, version.patch)
            }
        };
        let version = Version::new("MongoDB", version);
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
    logger: Logger,
    tracer: Option<Arc<Tracer>>,
}

impl Store {
    /// Create a mongodb-backed primary store interface.
    pub fn make<T>(config: MongoDBConfig, logger: Logger, tracer: T) -> Result<Store>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        info!(logger, "Instantiating primary store backed by MongoDB");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.uri.clone()))?;
        let tracer = tracer.into();
        Ok(Store {
            client,
            db,
            logger,
            tracer,
        })
    }
}

impl StoreInterface for Store {
    fn agent(&self) -> AgentImpl {
        let agent = self::agent::Agent::new(self.client.clone(), self.db.clone());
        AgentImpl::new(agent)
    }

    fn agents(&self) -> AgentsImpl {
        let agents = self::agents::Agents::new(self.client.clone(), self.db.clone());
        AgentsImpl::new(agents)
    }

    fn cluster(&self) -> ClusterImpl {
        let cluster = self::cluster::Cluster::new(
            self.client.clone(),
            self.db.clone(),
            self.logger.clone(),
            self.tracer.clone(),
        );
        ClusterImpl::new(cluster)
    }

    fn legacy(&self) -> LegacyImpl {
        let legacy = self::legacy::Legacy::new(self.client.clone(), self.db.clone());
        LegacyImpl::new(legacy)
    }

    fn node(&self) -> NodeImpl {
        let node = self::node::Node::new(self.client.clone(), self.db.clone());
        NodeImpl::new(node)
    }

    fn nodes(&self) -> NodesImpl {
        let nodes = self::nodes::Nodes::new(self.client.clone(), self.db.clone());
        NodesImpl::new(nodes)
    }

    fn persist(&self) -> PersistImpl {
        let persist =
            self::persist::Persist::new(self.client.clone(), self.db.clone(), self.tracer.clone());
        PersistImpl::new(persist)
    }

    fn shard(&self) -> ShardImpl {
        let shard = self::shard::Shard::new(self.client.clone(), self.db.clone());
        ShardImpl::new(shard)
    }

    fn shards(&self) -> ShardsImpl {
        let shards = self::shards::Shards::new(self.client.clone(), self.db.clone());
        ShardsImpl::new(shards)
    }
}
