use failure::ResultExt;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::Cursor;
use super::super::EventsFilters;
use super::super::EventsOptions;

use super::super::ErrorKind;
use super::super::Result;
use super::super::ValidationResult;
use super::super::config::MongoDBConfig;
use super::super::store::InnerStore;
use super::super::validator::InnerValidator;


mod constants;
mod metrics;

mod agent;
mod cluster;
mod datastore;
mod event;
mod validator;

pub use self::metrics::register_metrics;

use self::agent::AgentStore;
use self::datastore::DatastoreStore;
use self::cluster::ClusterStore;
use self::event::EventStore;

use self::validator::DataValidator;
use self::validator::IndexValidator;
use self::validator::SchemaValidator;


/// MongoDB-backed storage layer.
///
/// # Special collection requirements
///
///   * `events`: is a capped or TTL indexed collection.
///
/// # Expected indexes
///
///   * Index on `clusters_meta`: `(nodes: -1, name: 1)`
///   * Unique index on `agents`: `(cluster: 1, host: 1)`
///   * Unique index on `agents_info`: `(cluster: 1, host: 1)`
///   * Unique index on `clusters_meta`: `name: 1`
///   * Unique index on `discoveries`: `name: 1`
///   * Unique index on `nodes`: `(cluster: 1, name: 1)`
///   * Unique index on `shards`: `(cluster: 1, node: 1, id: 1)`
pub struct MongoStore {
    agents: AgentStore,
    clusters: ClusterStore,
    datastores: DatastoreStore,
    events: EventStore,
}

impl InnerStore for MongoStore {
    fn agent(&self, cluster: String, host: String) -> Result<Option<Agent>> {
        self.agents.agent(cluster, host)
    }

    fn agent_info(&self, cluster: String, host: String) -> Result<Option<AgentInfo>> {
        self.agents.agent_info(cluster, host)
    }

    fn cluster_agents(&self, cluster: String) -> Result<Cursor<Agent>> {
        self.agents.cluster_agents(cluster)
    }

    fn cluster_agents_info(&self, cluster: String) -> Result<Cursor<AgentInfo>> {
        self.agents.cluster_agents_info(cluster)
    }

    fn cluster_discovery(&self, cluster: String) -> Result<Option<ClusterDiscovery>> {
        self.clusters.cluster_discovery(cluster)
    }

    fn cluster_meta(&self, cluster: String) -> Result<Option<ClusterMeta>> {
        self.clusters.cluster_meta(cluster)
    }

    fn cluster_nodes(&self, cluster: String) -> Result<Cursor<Node>> {
        self.datastores.cluster_nodes(cluster)
    }

    fn cluster_shards(&self, cluster: String) -> Result<Cursor<Shard>> {
        self.datastores.cluster_shards(cluster)
    }

    fn events(&self, filters: EventsFilters, options: EventsOptions) -> Result<Cursor<Event>> {
        self.events.events(filters, options)
    }

    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<ClusterMeta>> {
        self.clusters.find_clusters(&search, limit)
    }

    fn node(&self, cluster: String, name: String) -> Result<Option<Node>> {
        self.datastores.node(cluster, name)
    }

    fn persist_agent(&self, agent: Agent) -> Result<()> {
        self.agents.persist_agent(agent)
    }

    fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        self.agents.persist_agent_info(agent)
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        self.clusters.persist_cluster_meta(meta)
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        self.clusters.persist_discovery(cluster)
    }

    fn persist_event(&self, event: Event) -> Result<()> {
        self.events.persist_event(event)
    }

    fn persist_node(&self, node: Node) -> Result<()> {
        self.datastores.persist_node(node)
    }

    fn persist_shard(&self, shard: Shard) -> Result<()> {
        self.datastores.persist_shard(shard)
    }

    fn shard(&self, cluster: String, node: String, id: String) -> Result<Option<Shard>> {
        self.datastores.shard(cluster, node, id)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        self.clusters.top_clusters()
    }
}

impl MongoStore {
    /// Creates a mongodb-backed store.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn new(config: MongoDBConfig, logger: Logger) -> Result<MongoStore> {
        info!(logger, "Configuring MongoDB as storage layer");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.uri.clone()))?;
        let agents = AgentStore::new(client.clone(), db.clone());
        let clusters = ClusterStore::new(client.clone(), db.clone(), logger);
        let datastores = DatastoreStore::new(client.clone(), db.clone());
        let events = EventStore::new(client, db);
        Ok(MongoStore {
            agents,
            clusters,
            datastores,
            events,
        })
    }
}


/// MongoDB-backed storage validator.
pub struct MongoValidator {
    client: Client,
    db: String,
    data: DataValidator,
    index: IndexValidator,
    schema: SchemaValidator,
}

impl InnerValidator for MongoValidator {
    fn agents(&self) -> Result<Cursor<Agent>> {
        self.data.agents()
    }

    fn agents_count(&self) -> Result<u64> {
        self.data.agents_count()
    }

    fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        self.data.agents_info()
    }

    fn agents_info_count(&self) -> Result<u64> {
        self.data.agents_info_count()
    }

    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        self.data.cluster_discoveries()
    }

    fn cluster_discoveries_count(&self) -> Result<u64> {
        self.data.cluster_discoveries_count()
    }

    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        self.data.clusters_meta()
    }

    fn clusters_meta_count(&self) -> Result<u64> {
        self.data.clusters_meta_count()
    }

    fn events(&self) -> Result<Cursor<Event>> {
        self.data.events()
    }

    fn events_count(&self) -> Result<u64> {
        self.data.events_count()
    }

    fn indexes(&self) -> Result<Vec<ValidationResult>> {
        self.index.indexes()
    }

    fn nodes(&self) -> Result<Cursor<Node>> {
        self.data.nodes()
    }

    fn nodes_count(&self) -> Result<u64> {
        self.data.nodes_count()
    }

    fn removed(&self) -> Result<Vec<ValidationResult>> {
        // There is nothing removed yet.
        Ok(vec![])
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        self.schema.schema()
    }

    fn shards(&self) -> Result<Cursor<Shard>> {
        self.data.shards()
    }

    fn shards_count(&self) -> Result<u64> {
        self.data.shards_count()
    }

    fn version(&self) -> Result<String> {
        let db = self.client.db(&self.db);
        let version = db.version().with_context(|_| ErrorKind::MongoDBOperation("version"))?;
        let version = format!("MongoDB {}", version);
        Ok(version)
    }
}

impl MongoValidator {
    /// Creates a mongodb-backed store validator.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn new(config: MongoDBConfig, logger: Logger, registry: &Registry) -> Result<MongoValidator> {
        info!(logger, "Configuring MongoDB as storage validator");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri)
            .with_context(|_| ErrorKind::MongoDBConnect(config.uri.clone()))?;
        let data = DataValidator::new(db.clone(), client.clone());
        let index = IndexValidator::new(db.clone(), client.clone());
        let schema = SchemaValidator::new(db.clone(), client.clone());

        register_metrics(&logger, registry);
        Ok(MongoValidator {
            client, db,
            data, index, schema
        })
    }
}
