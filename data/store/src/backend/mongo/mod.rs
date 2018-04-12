use bson;
use bson::Bson;
use bson::Document;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;

use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;
use regex;
use slog::Logger;

use replicante_data_models::Cluster;
use replicante_data_models::Node;

use replicante_data_models::webui::ClusterListItem;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


static COLLECTION_CLUSTERS: &'static str = "clusters";
static COLLECTION_NODES: &'static str = "nodes";

static FAIL_CLIENT: &'static str = "Failed to configure MongoDB client";
static FAIL_FIND_CLUSTERS: &'static str = "Failed while searching for clusters";
static FAIL_PERSIST_CLUSTER: &'static str = "Failed to persist cluster";
static FAIL_PERSIST_NODE: &'static str = "Failed to persist node";
static FAIL_TOP_CLUSTERS: &'static str = "Failed to list biggest clusters";

static TOP_CLUSTERS_LIMIT: u32 = 10;


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
///
///   * Unique index on `clusters`: `name`
///   * Unique index on `nodes`: `(info.agent.cluster, info.agent.name)`
pub struct MongoStore {
    db: String,
    client: Client,
}

impl InnerStore for MongoStore {
    fn find_clusters(&self, search: String, limit: u8) -> Result<Vec<String>> {
        let search = regex::escape(&search);
        let filter = doc!{"name" => {"$regex" => search, "$options" => "i"}};
        let mut options = FindOptions::new();
        options.limit = Some(limit as i64);
        options.projection = Some(doc!{"name" => 1});

        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_clusters();
        let cursor = collection.find(Some(filter), Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_FIND_CLUSTERS)?;
            let doc = bson::from_bson::<FindClustersResult>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_FIND_CLUSTERS)?;
            clusters.push(doc.name);
        }
        Ok(clusters)
    }

    fn fetch_top_clusters(&self) -> Result<Vec<ClusterListItem>> {
        let group = doc! {
            "$group" => {
                "_id" => "$info.datastore.cluster",
                "nodes" => {"$sum" => 1},
                "kinds" => {"$addToSet" => "$info.datastore.kind"},
            }
        };
        let sort = doc! {"$sort" => {"nodes" => 1}};
        let limit = doc! {"$limit" => TOP_CLUSTERS_LIMIT};
        let project = doc! {
            "$project" => {
                "_id" => 0,
                "name" => "$_id",
                "kinds" => 1,
                "nodes" => 1,
            }
        };
        MONGODB_OPS_COUNT.with_label_values(&["aggregate"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["aggregate"]).start_timer();
        self.process_top_clusters(vec![group, sort, limit, project])
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["aggregate"]).inc();
                error
            })
            .chain_err(|| FAIL_TOP_CLUSTERS)
    }

    fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>> {
        let replacement = bson::to_bson(&cluster).chain_err(|| FAIL_PERSIST_CLUSTER)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Cluster failed to encode as BSON document")
        };
        let filter = doc!{"name" => cluster.name};
        let collection = self.collection_clusters();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER)?;
        match old {
            None => Ok(None),
            Some(doc) => Ok(Some(bson::from_bson::<Cluster>(Bson::Document(doc))?))
        }
    }

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
        let collection = self.collection_nodes();
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

    /// Runs the aggregation pipeline and converts the results.
    fn process_top_clusters(&self, steps: Vec<Document>) -> Result<Vec<ClusterListItem>> {
        let collection = self.collection_nodes();
        let cursor = collection.aggregate(steps, None)?;
        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc?;
            let doc = bson::from_bson::<ClusterListItem>(bson::Bson::Document(doc))?;
            clusters.push(doc);
        }
        Ok(clusters)
    }

    /// Returns the collection storing clusters.
    fn collection_clusters(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_CLUSTERS)
    }

    /// Returns the collection storing nodes.
    fn collection_nodes(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }
}


/// Internal structure for `MongoStore::find_clusters` result.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct FindClustersResult {
    name: String,
}
