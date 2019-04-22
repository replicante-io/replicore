use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Shard;

use super::super::super::store::shards::ShardsAttribures;
use super::super::super::Cursor;
use super::super::super::Result;
use super::super::ShardsInterface;
use super::common::find;
use super::constants::COLLECTION_SHARDS;
use super::document::ShardDocument;

/// Shards operations implementation using MongoDB.
pub struct Shards {
    client: Client,
    db: String,
}

impl Shards {
    pub fn new(client: Client, db: String) -> Shards {
        Shards { client, db }
    }
}

impl ShardsInterface for Shards {
    fn iter(&self, attrs: &ShardsAttribures) -> Result<Cursor<Shard>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let cursor = find(collection, filter)?
            .map(|result: Result<ShardDocument>| result.map(Shard::from));
        Ok(Cursor(Box::new(cursor)))
    }
}
