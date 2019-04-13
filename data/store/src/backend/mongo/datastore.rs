use bson;
use bson::Bson;
use failure::ResultExt;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::UpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;

use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;

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

    pub fn cluster_nodes(&self, cluster_id: String) -> Result<Cursor<Node>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_nodes();
        let cursor = collection.find(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        timer.observe_duration();
        let iter = cursor.map(|doc| {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let node = bson::from_bson::<Node>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            Ok(node.into())
        });
        Ok(Cursor(Box::new(iter)))
    }

    pub fn cluster_shards(&self, cluster_id: String) -> Result<Cursor<Shard>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_shards();
        let cursor = collection.find(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        timer.observe_duration();
        let iter = cursor.map(|doc| {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let shard = bson::from_bson::<Shard>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            Ok(shard.into())
        });
        Ok(Cursor(Box::new(iter)))
    }

    pub fn node(&self, cluster_id: String, node_id: String) -> Result<Option<Node>> {
        let filter = doc!{
            "cluster_id" => cluster_id,
            "node_id" => node_id,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_nodes();
        let node = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if node.is_none() {
            return Ok(None);
        }
        let node = node.unwrap();
        let node = bson::from_bson::<Node>(bson::Bson::Document(node))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
        Ok(Some(node))
    }

    pub fn persist_node(&self, node: Node) -> Result<()> {
        let replacement = bson::to_bson(&node).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Node failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster_id" => node.cluster_id,
            "node_id" => node.node_id,
        };
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        let collection = self.collection_nodes();
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        Ok(())
    }

    pub fn persist_shard(&self, shard: Shard) -> Result<()> {
        let replacement = bson::to_bson(&shard).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Shard failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster_id" => shard.cluster_id,
            "node_id" => shard.node_id,
            "shard_id" => shard.shard_id,
        };
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        let collection = self.collection_shards();
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        Ok(())
    }

    pub fn shard(
        &self,
        cluster_id: String,
        node_id: String,
        shard_id: String,
    ) -> Result<Option<Shard>> {
        let filter = doc!{
            "cluster_id" => cluster_id,
            "node_id" => node_id,
            "shard_id" => shard_id,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_shards();
        let shard = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if shard.is_none() {
            return Ok(None);
        }
        let shard = shard.unwrap();
        let shard = bson::from_bson::<Shard>(bson::Bson::Document(shard))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
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
