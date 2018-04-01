use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::db::ThreadedDatabase;

use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Node;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


static COLLECTION_NODES: &'static str = "nodes";
static FAIL_CLIENT: &'static str = "Failed to configure MongoDB client";
static FAIL_PERSIST_NODE: &'static str = "Failed to persist node";


lazy_static! {
    /// Counter for MongoDB operations.
    static ref MONGODB_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_mongodb_operations", "Number of MongoDB operations issued"),
        &["operation"]
    ).expect("Failed to create replicante_mongodb_operations counter");

    /// Counter for MongoDB operation errors.
    static ref MONGODB_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_mongodb_operation_errors", "Number of MongoDB operations failed"),
        &["operation"]
    ).expect("Failed to create replicante_mongodb_operation_errors counter");

    /// Observe duration of MongoDB operations.
    static ref MONGODB_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicante_mongodb_operations_duration",
            "Duration (in seconds) of MongoDB operations"
        ),
        &["operation"]
    ).expect("Failed to create MONGODB_OPS_DURATION histogram");
}


/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(MONGODB_OPS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MONGODB_OPS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(MONGODB_OP_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MONGODB_OP_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(MONGODB_OPS_DURATION.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register MONGODB_OPS_DURATION"; "error" => error);
    }
}


/// MongoDB-backed storage layer.
///
/// # Special collection requirements
///
///   * `events`: capped collection or TTL indexed.
///
/// # Expected indexes
/// ## `nodes` collection
///   * Unique index on `(info.agent.cluster, info.agent.name)`
pub struct MongoStore {
    db: String,
    client: Client,
}

impl InnerStore for MongoStore {
    fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        let replacement = bson::to_bson(&node).chain_err(|| FAIL_PERSIST_NODE)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Node failed to encode as BSON document")
        };
        let filter = doc!{
            "info.datastore.cluster" => node.info.datastore.cluster,
            "info.datastore.name" => node.info.datastore.name,
        };
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        let collection = self.nodes_collection();
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_NODE)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let old: Node = bson::from_bson(Bson::Document(doc))?;
                Ok(Some(old))
            }
        }
    }
}

impl MongoStore {
    /// Creates a mongodb-backed store.
    pub fn new(config: MongoDBConfig, logger: Logger, registry: &Registry) -> Result<MongoStore> {
        info!(logger, "Configuring MongoDB as storage layer");
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri).chain_err(|| FAIL_CLIENT)?;

        register_metrics(&logger, registry);
        Ok(MongoStore {
            db,
            client,
        })
    }

    /// Returns the collection storing nodes.
    fn nodes_collection(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }
}
