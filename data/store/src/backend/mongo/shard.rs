use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Shard as ShardModel;

use super::super::super::store::shard::ShardAttribures;
use super::super::super::Result;
use super::super::ShardInterface;
use super::common::find_one;
use super::constants::COLLECTION_SHARDS;
use super::document::ShardDocument;

/// Shard operations implementation using MongoDB.
pub struct Shard {
    client: Client,
    db: String,
}

impl Shard {
    pub fn new(client: Client, db: String) -> Shard {
        Shard { client, db }
    }
}

impl ShardInterface for Shard {
    fn get(&self, attrs: &ShardAttribures) -> Result<Option<ShardModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &attrs.node_id,
            "shard_id" => &attrs.shard_id,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let document: Option<ShardDocument> = find_one(collection, filter)?;
        Ok(document.map(ShardModel::from))
    }
}
