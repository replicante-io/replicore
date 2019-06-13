use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::ordered::OrderedDocument;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_models_core::Shard;

use super::super::super::store::shards::ShardsAttribures;
use super::super::super::store::shards::ShardsCounts;
use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::ShardsInterface;
use super::common::aggregate;
use super::common::find;
use super::constants::COLLECTION_SHARDS;
use super::document::ShardDocument;

/// Return a document to count shards in given state as part of the $group stage.
fn aggregate_count_role(role: &'static str) -> OrderedDocument {
    doc! {"$sum" => {
        "$cond" => {
            "if" => {"$eq" => ["$role", role]},
            "then" => 1,
            "else" => 0,
        }
    }}
}

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
    fn counts(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<ShardsCounts> {
        // Let mongo figure out the counts with an aggregation.
        // Remember to count each shard only once across all nodes (and NOT once per node).
        let filter = doc! {"$match" => {
            "cluster_id" => &attrs.cluster_id,
            "stale" => false,
        }};
        let count_nodes = doc! {"$sum" => 1};
        let count_primaries = aggregate_count_role("primary");
        // First aggregate counts for each shard.
        let group_map = doc! {"$group" => {
            "_id" => {
                "cluster_id" => "$cluster_id",
                "shard_id" => "$shard_id",
            },
            "nodes" => count_nodes,
            "primaries" => count_primaries,
        }};
        // Then aggregate all shards into one document.
        let group_reduce = doc! {"$group" => {
            "_id" => "$cluster_id",
            "shards" => {"$sum" => 1},
            "primaries" => {"$sum" => "$primaries"},
        }};
        let pipeline = vec![filter, group_map, group_reduce];

        // Run aggrgation and grab the one and only (expected) result.
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let mut cursor = aggregate(
            collection,
            pipeline,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?;
        let counts: ShardsCounts = match cursor.next() {
            Some(counts) => counts?,
            None => {
                return Ok(ShardsCounts {
                    shards: 0,
                    primaries: 0,
                })
            }
        };
        if cursor.next().is_some() {
            return Err(
                ErrorKind::DuplicateRecord("ShardsCounts", attrs.cluster_id.clone()).into(),
            );
        }
        Ok(counts)
    }

    fn iter(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<Cursor<Shard>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let cursor = find(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?
        .map(|result: Result<ShardDocument>| result.map(Shard::from));
        Ok(Cursor(Box::new(cursor)))
    }
}
