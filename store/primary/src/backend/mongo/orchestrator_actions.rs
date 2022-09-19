use std::sync::Arc;

use bson::doc;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_models_core::actions::orchestrator::OrchestratorActionSyncSummary;
use replicante_models_core::api::orchestrator_action::OrchestratorActionSummary;

use super::super::OrchestratorActionsInterface;
use super::constants::COLLECTION_ACTIONS_ORCHESTRATOR;
use super::document::OrchestratorActionSummaryDocument;
use crate::store::orchestrator_actions::OrchestratorActionsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Orchestrator actions operations implementation using MongoDB.
pub struct OrchestratorActions {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl OrchestratorActions {
    pub fn new<T>(client: Client, db: String, tracer: T) -> OrchestratorActions
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        OrchestratorActions { client, db, tracer }
    }
}

impl OrchestratorActionsInterface for OrchestratorActions {
    fn iter_summary(
        &self,
        attrs: &OrchestratorActionsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<OrchestratorActionSummary>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let mut options = FindOptions::default();
        options.projection = Some(doc! {
            "cluster_id": 1,
            "action_id": 1,
            "created_ts": 1,
            "finished_ts": 1,
            "kind": 1,
            "state": 1,
        });
        options.sort = Some(doc! {
            "cluster_id": 1,
            "action_id": 1,
            "created_ts": -1,
        });
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let cursor = find_with_options(collection, filter, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let cursor = cursor.map(|document| {
            let document: OrchestratorActionSummaryDocument =
                document.with_context(|_| ErrorKind::MongoDBCursor)?;
            Ok(OrchestratorActionSummary::from(document))
        });
        Ok(Cursor::new(cursor))
    }

    fn unfinished_summaries(
        &self,
        attrs: &OrchestratorActionsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<OrchestratorActionSyncSummary>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "finished_ts": null,
        };
        let mut options = FindOptions::default();
        options.projection = Some(doc! {
            "cluster_id": 1,
            "action_id": 1,
            "kind": 1,
            "state": 1,
        });
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let cursor = find_with_options(collection, filter, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
