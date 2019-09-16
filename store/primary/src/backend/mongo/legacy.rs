use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::Bson;
use failure::Fail;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use regex;

use replicante_externals_mongodb::operations::find_one;
use replicante_externals_mongodb::operations::find_with_options;
use replicante_externals_mongodb::operations::replace_one;
use replicante_models_core::cluster::ClusterMeta;

use super::super::LegacyInterface;
use super::constants::COLLECTION_CLUSTER_META;
use super::constants::TOP_CLUSTERS_LIMIT;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Legacy operations implementation using MongoDB.
pub struct Legacy {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Legacy {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Legacy
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Legacy { client, db, tracer }
    }
}

impl LegacyInterface for Legacy {
    fn cluster_meta(
        &self,
        cluster_id: String,
        span: Option<SpanContext>,
    ) -> Result<Option<ClusterMeta>> {
        let filter = doc! {"cluster_id" => &cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let meta = find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(meta)
    }

    fn find_clusters(
        &self,
        search: String,
        limit: u8,
        span: Option<SpanContext>,
    ) -> Result<Cursor<ClusterMeta>> {
        let search = regex::escape(&search);
        let filter = doc! { "$or" => [
            {"cluster_display_name" => {"$regex" => &search, "$options" => "i"}},
            {"cluster_id" => {"$regex" => &search, "$options" => "i"}},
        ]};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(limit));
        let cursor = find_with_options(
            collection,
            filter,
            options,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?
        .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {"cluster_id" => &meta.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let document = bson::to_bson(&meta).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("ClusterMeta failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn top_clusters(&self, span: Option<SpanContext>) -> Result<Cursor<ClusterMeta>> {
        let filter = doc! {};
        let sort = doc! {
            "shards" => -1,
            "nodes" => -1,
            "cluster_id" => 1,
        };
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(TOP_CLUSTERS_LIMIT));
        options.sort = Some(sort);
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let cursor = find_with_options(
            collection,
            filter,
            options,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?
        .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
