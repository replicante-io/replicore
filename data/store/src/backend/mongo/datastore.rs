use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Node;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_NODES;
use super::constants::FAIL_PERSIST_NODE;

use super::metrics::MONGODB_OP_ERRORS_COUNT;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;


/// Subset of the `Store` trait that deals with nodes.
pub struct NodeStore {
    client: Client,
    db: String,
}

impl NodeStore {
    pub fn new(client: Client, db: String) -> NodeStore {
        NodeStore { client, db }
    }

    pub fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        let replacement = bson::to_bson(&node).chain_err(|| FAIL_PERSIST_NODE)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Node failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster" => node.cluster,
            "name" => node.name,
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
                Ok(Some(bson::from_bson(Bson::Document(doc))?))
            }
        }
    }

    /// Returns the collection storing nodes.
    fn collection_nodes(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }
}
