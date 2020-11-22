use std::sync::Arc;

use bson::doc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_externals_mongodb::operations::find_one;
use replicante_externals_mongodb::operations::update_many;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use super::super::ClusterInterface;
use super::constants::COLLECTION_CLUSTER_SETTINGS;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::STALE_COLLECTIONS;
use crate::store::cluster::ClusterAttribures;
use crate::ErrorKind;
use crate::Result;

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
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_DISCOVERIES);
        let discovery = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(discovery)
    }

    fn mark_stale(&self, attrs: &ClusterAttribures, span: Option<SpanContext>) -> Result<()> {
        for name in STALE_COLLECTIONS.iter() {
            let collection = self.client.database(&self.db).collection(name);
            let filter = doc! {"cluster_id": &attrs.cluster_id};
            let mark = doc! {"$set": {"stale": true}};
            let stats = update_many(
                collection,
                filter,
                mark,
                span.clone(),
                self.tracer.as_deref(),
            )
            .with_context(|_| ErrorKind::MongoDBOperation)?;
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

    fn settings(
        &self,
        attrs: &ClusterAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<ClusterSettings>> {
        let filter = doc! {
            "namespace": &attrs.namespace,
            "cluster_id": &attrs.cluster_id,
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_CLUSTER_SETTINGS);
        let settings = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(settings)
    }
}
