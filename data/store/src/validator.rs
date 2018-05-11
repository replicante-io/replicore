use std::sync::Arc;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterMeta;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::Config;
use super::Cursor;
use super::Result;

use super::backend::mongo::MongoValidator;


/// Private interface to the persistence storage validation.
///
/// Allows multiple possible datastores to be used as well as mocks for testing.
pub trait InnerValidator: Send + Sync {
    /// See `Validator::agents` for details.
    fn agents(&self) -> Result<Cursor<Agent>>;

    /// See `Validator::agents_count` for details.
    fn agents_count(&self) -> Result<u64>;

    /// See `Validator::agents_info` for details.
    fn agents_info(&self) -> Result<Cursor<AgentInfo>>;

    /// See `Validator::agents_info_count` for details.
    fn agents_info_count(&self) -> Result<u64>;

    /// See `Validator::cluster_discoveries` for details.
    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>>;

    /// See `Validator::cluster_discoveries_count` for details.
    fn cluster_discoveries_count(&self) -> Result<u64>;

    /// See `Validator::clusters_meta` for details.
    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>>;

    /// See `Validator::clusters_meta_count` for details.
    fn clusters_meta_count(&self) -> Result<u64>;

    /// See `Validator::events` for details.
    fn events(&self) -> Result<Cursor<Event>>;

    /// See `Validator::events_count` for details.
    fn events_count(&self) -> Result<u64>;

    /// See `Validator::indexes` for details.
    fn indexes(&self) -> Result<Vec<ValidationResult>>;

    /// See `Validator::nodes` for details.
    fn nodes(&self) -> Result<Cursor<Node>>;

    /// See `Validator::nodes_count` for details.
    fn nodes_count(&self) -> Result<u64>;

    /// See `Validator::removed` for details.
    fn removed(&self) -> Result<Vec<ValidationResult>>;

    /// See `Validator::schema` for details.
    fn schema(&self) -> Result<Vec<ValidationResult>>;

    /// See `Validator::shards` for details.
    fn shards(&self) -> Result<Cursor<Shard>>;

    /// See `Validator::shards_count` for details.
    fn shards_count(&self) -> Result<u64>;
}


/// Details of issues detected by the validation process.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ValidationResult {
    pub collection: String,
    pub error: bool,
    pub group: &'static str,
    pub message: String,
}

impl ValidationResult {
    /// Create a `ValidationResult` for an error.
    pub fn error<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
        where S1: Into<String>,
              S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: true,
            group,
            message: message.into(),
        }
    }

    /// Create a `ValidationResult` for a non-critical issue or a suggestion.
    pub fn result<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
        where S1: Into<String>,
              S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: false,
            group,
            message: message.into(),
        }
    }
}


/// Public interface to the persistent storage validation.
///
/// This interface abstracts away details about access to stored models to allow
/// for validation logic to be implemented on top of any supported datastore.
#[derive(Clone)]
pub struct Validator(Arc<InnerValidator>);

impl Validator {
    /// Instantiate a new storage validator.
    pub fn new(config: Config, logger: Logger, registry: &Registry) -> Result<Validator> {
        let validator = match config {
            Config::MongoDB(config) => Arc::new(MongoValidator::new(config, logger, registry)?),
        };
        Ok(Validator(validator))
    }

    /// Iterate over stored agents.
    pub fn agents(&self) -> Result<Cursor<Agent>> {
        self.0.agents()
    }

    /// Approximate count of agents in the store.
    pub fn agents_count(&self) -> Result<u64> {
        self.0.agents_count()
    }

    /// Iterate over stored agents info.
    pub fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        self.0.agents_info()
    }

    /// Approximate count of agents info in the store.
    pub fn agents_info_count(&self) -> Result<u64> {
        self.0.agents_info_count()
    }

    /// Iterate over stored cluster discoveries.
    pub fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        self.0.cluster_discoveries()
    }

    /// Approximate count of cluster discoveries in the store.
    pub fn cluster_discoveries_count(&self) -> Result<u64> {
        self.0.cluster_discoveries_count()
    }

    /// Iterate over stored cluster meta.
    pub fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        self.0.clusters_meta()
    }

    /// Approximate count of cluster meta in the store.
    pub fn clusters_meta_count(&self) -> Result<u64> {
        self.0.clusters_meta_count()
    }

    /// Iterate over stored events.
    pub fn events(&self) -> Result<Cursor<Event>> {
        self.0.events()
    }

    /// Approximate count of events in the store.
    pub fn events_count(&self) -> Result<u64> {
        self.0.events_count()
    }

    /// Validate the current indexes to ensure they matches the code.
    pub fn indexes(&self) -> Result<Vec<ValidationResult>> {
        self.0.indexes()
    }

    /// Iterate over stored nodes.
    pub fn nodes(&self) -> Result<Cursor<Node>> {
        self.0.nodes()
    }

    /// Approximate count of nodes in the store.
    pub fn nodes_count(&self) -> Result<u64> {
        self.0.nodes_count()
    }

    /// Checks the store for collections/tables or indexes that are no longer used.
    pub fn removed(&self) -> Result<Vec<ValidationResult>> {
        self.0.removed()
    }

    /// Validate the current schema to ensure it matches the code.
    pub fn schema(&self) -> Result<Vec<ValidationResult>> {
        self.0.schema()
    }

    /// Iterate over stored shards.
    pub fn shards(&self) -> Result<Cursor<Shard>> {
        self.0.shards()
    }

    /// Approximate count of shards in the store.
    pub fn shards_count(&self) -> Result<u64> {
        self.0.shards_count()
    }

    /// Instantiate a `Validator` that wraps the given `MockValidator`.
    // Cargo builds dependencies in debug mode instead of test mode.
    // That means that `cfg(test)` cannot be used if the mock is used outside the crate.
    #[cfg(debug_assertions)]
    pub fn mock(inner: Arc<super::mock::MockValidator>) -> Validator {
        Validator(inner)
    }
}
