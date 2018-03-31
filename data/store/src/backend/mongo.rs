use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Node;

use super::super::InnerStore;
use super::super::Result;
use super::super::ResultExt;
use super::super::config::MongoDBConfig;


static COLLECTION_NODES: &'static str = "nodes";
static FAIL_PERSIST_NODE: &'static str = "Failed to persist node";


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
    // TODO: update this method once the agent returns the cluster ID.
    fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        let replacement = bson::to_bson(&node).chain_err(|| FAIL_PERSIST_NODE)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Node failed to encode as BSON document")
        };
        let filter = doc!{
            //"info.datastore.cluster" => "TODO",
            "info.datastore.name" => node.info.datastore.name
        };
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        let collection = self.nodes_collection();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
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
    pub fn new(config: MongoDBConfig) -> Result<MongoStore> {
        let db = config.db.clone();
        let client = Client::with_uri(&config.uri)?;
        Ok(MongoStore { db, client })
    }

    /// Returns the collection storing nodes.
    fn nodes_collection(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }
}
