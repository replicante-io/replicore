use std::sync::Arc;

use bson::doc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::cluster::OrchestrateReport;

use super::super::ClusterInterface;
use super::constants::COLLECTION_CLUSTER_ORCHESTRATE_REPORT;
use super::document::OrchestrateReportDocument;
use crate::store::cluster::ClusterAttributes;
use crate::ErrorKind;
use crate::Result;

/// Cluster operations implementation using MongoDB.
pub struct Cluster {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Cluster {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Cluster
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Cluster { client, db, tracer }
    }
}

impl ClusterInterface for Cluster {
    fn orchestrate_report(
        &self,
        attrs: &ClusterAttributes,
        span: Option<SpanContext>,
    ) -> Result<Option<OrchestrateReport>> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_CLUSTER_ORCHESTRATE_REPORT);
        let filter = doc! {
            "namespace": attrs.namespace,
            "cluster_id": attrs.cluster_id,
        };
        let report: Option<OrchestrateReportDocument> =
            find_one(collection, filter, span, self.tracer.as_deref())
                .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(report.map(OrchestrateReport::from))
    }
}
