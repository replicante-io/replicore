use std::sync::Arc;

use failure::Fail;
use failure::ResultExt;
use mongodb::bson::doc;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Shard;

use super::super::ShardsInterface;
use super::constants::COLLECTION_SHARDS;
use crate::store::shards::ShardsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Shards operations implementation using MongoDB.
pub struct Shards {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Shards {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Shards
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Shards { client, db, tracer }
    }
}

impl ShardsInterface for Shards {
    fn iter(&self, attrs: &ShardsAttributes, span: Option<SpanContext>) -> Result<Cursor<Shard>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self.client.database(&self.db).collection(COLLECTION_SHARDS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
