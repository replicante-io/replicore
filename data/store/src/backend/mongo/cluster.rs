use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_data_models::ClusterDiscovery;

use super::super::super::store::cluster::ClusterAttribures;
use super::super::super::Result;
use super::super::ClusterInterface;
use super::common::find_one;
use super::common::update_many;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::STALE_COLLECTIONS;

/// Clusters operations implementation using MongoDB.
pub struct Cluster {
    client: Client,
    db: String,
    logger: Logger,
    tracer: Option<Arc<Tracer>>,
}

impl Cluster {
    pub fn new<T>(client: Client, db: String, logger: Logger, tracer: T) -> Cluster
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Cluster {
            client,
            db,
            logger,
            tracer,
        }
    }
}

impl ClusterInterface for Cluster {
    fn discovery(
        &self,
        attrs: &ClusterAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<ClusterDiscovery>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_DISCOVERIES);
        find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn mark_stale(&self, attrs: &ClusterAttribures, span: Option<SpanContext>) -> Result<()> {
        for name in STALE_COLLECTIONS.iter() {
            let collection = self.client.db(&self.db).collection(name);
            let filter = doc! {"cluster_id" => &attrs.cluster_id};
            let mark = doc! {"$set" => {"stale" => true}};
            let stats = update_many(
                collection,
                filter,
                mark,
                span.clone(),
                self.tracer.as_ref().map(|tracer| tracer.deref()),
            )?;
            debug!(
                self.logger, "Marked cluster as stale";
                "cluster_id" => &attrs.cluster_id,
                "collection" => name,
                "matched_count" => stats.matched_count,
                "modified_count" => stats.modified_count,
            );
        }
        Ok(())
    }
}
