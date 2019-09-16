use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::agent::Shard as ShardModel;

use super::super::ShardInterface;
use super::constants::COLLECTION_SHARDS;
use super::document::ShardDocument;
use crate::store::shard::ShardAttribures;
use crate::ErrorKind;
use crate::Result;

/// Shard operations implementation using MongoDB.
pub struct Shard {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Shard {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Shard
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Shard { client, db, tracer }
    }
}

impl ShardInterface for Shard {
    fn get(
        &self,
        attrs: &ShardAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<ShardModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &attrs.node_id,
            "shard_id" => &attrs.shard_id,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let document: Option<ShardDocument> = find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(document.map(ShardModel::from))
    }
}
