use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::constants::FAIL_FIND_NODE;
use super::constants::FAIL_FIND_SHARD;
use super::constants::FAIL_PERSIST_NODE;
use super::constants::FAIL_PERSIST_SHARD;

use super::metrics::MONGODB_OP_ERRORS_COUNT;
use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;


/// Subset of the `Store` trait that deals with nodes.
pub struct DatastoreStore {
    client: Client,
    db: String,
}

impl DatastoreStore {
    pub fn new(client: Client, db: String) -> DatastoreStore {
        DatastoreStore { client, db }
    }

    pub fn node(&self, cluster: String, name: String) -> Result<Option<Node>> {
        let filter = doc!{
            "cluster" => cluster,
            "name" => name,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_nodes();
        let node = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_NODE)?;
        timer.observe_duration();
        if node.is_none() {
            return Ok(None);
        }
        let node = node.unwrap();
        let node = bson::from_bson::<Node>(bson::Bson::Document(node))
            .chain_err(|| FAIL_FIND_NODE)?;
        Ok(Some(node))
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
                let node = bson::from_bson::<Node>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_NODE)?;
                Ok(Some(node))
            }
        }
    }

    pub fn persist_shard(&self, shard: Shard) -> Result<Option<Shard>> {
        let replacement = bson::to_bson(&shard).chain_err(|| FAIL_PERSIST_SHARD)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Shard failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster" => shard.cluster,
            "node" => shard.node,
            "id" => shard.id,
        };
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        let collection = self.collection_shards();
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_SHARD)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let shard = bson::from_bson::<Shard>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_SHARD)?;
                Ok(Some(shard))
            }
        }
    }

    pub fn shard(&self, cluster: String, node: String, id: String) -> Result<Option<Shard>> {
        let filter = doc!{
            "cluster" => cluster,
            "node" => node,
            "id" => id,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_shards();
        let shard = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_SHARD)?;
        timer.observe_duration();
        if shard.is_none() {
            return Ok(None);
        }
        let shard = shard.unwrap();
        let shard = bson::from_bson::<Shard>(bson::Bson::Document(shard))
            .chain_err(|| FAIL_FIND_SHARD)?;
        Ok(Some(shard))
    }

    /// Returns the collection storing nodes.
    fn collection_nodes(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_NODES)
    }

    /// Returns the collection storing shards.
    fn collection_shards(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_SHARDS)
    }
}
