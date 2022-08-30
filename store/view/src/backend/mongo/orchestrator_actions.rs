use std::sync::Arc;

use bson::doc;
use bson::Bson;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find_one;
use replicante_externals_mongodb::operations::find_with_options;
use replicante_models_core::actions::orchestrator::OrchestratorAction;

use super::super::OrchestratorActionsInterface;
use super::constants::COLLECTION_ACTIONS_ORCHESTRATOR;
use super::constants::MAX_ACTIONS_SEARCH;
use super::document::OrchestratorActionDocument;
use crate::store::orchestrator_actions::SearchFilters;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Actions operations implementation using MongoDB.
pub struct OrchestratorActions {
    client: Client,
    cluster_id: String,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl OrchestratorActions {
    pub fn new<T>(client: Client, db: String, tracer: T, cluster_id: String) -> OrchestratorActions
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        OrchestratorActions {
            client,
            cluster_id,
            db,
            tracer,
        }
    }
}

impl OrchestratorActionsInterface for OrchestratorActions {
    fn orchestrator_action(
        &self,
        action_id: Uuid,
        span: Option<SpanContext>,
    ) -> Result<Option<OrchestratorAction>> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let filter = doc! {
            "cluster_id": &self.cluster_id,
            "action_id": action_id.to_string(),
        };
        let action: Option<OrchestratorActionDocument> =
            find_one(collection, filter, span, self.tracer.as_deref())
                .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(action.map(OrchestratorAction::from))
    }

    fn search(
        &self,
        search: SearchFilters,
        span: Option<SpanContext>,
    ) -> Result<Cursor<OrchestratorAction>> {
        // Prepare options.
        let mut options = FindOptions::default();
        options.limit = Some(MAX_ACTIONS_SEARCH);
        options.sort = Some(doc! {"created_ts": -1});

        // Apply filters.
        let from = search.from;
        let until = search.until;
        let mut filters = vec![
            Bson::from(doc! {"cluster_id": {"$eq": &self.cluster_id}}),
            Bson::from(doc! {"created_ts": {"$gte": from}}),
            Bson::from(doc! {"created_ts": {"$lte": until}}),
        ];
        if let Some(action_kind) = search.action_kind {
            let action_kind = regex::escape(&action_kind);
            filters.push(Bson::from(doc! {"kind": {"$regex": action_kind}}));
        }
        if let Some(action_state) = search.action_state {
            let action_state = regex::escape(&action_state);
            filters.push(Bson::from(doc! {"state": {"$regex": action_state}}));
        }
        let filters = doc! {"$and": filters};

        // Execute the query.
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let cursor = find_with_options(collection, filters, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|item: Result<OrchestratorActionDocument>| item.map(OrchestratorAction::from));
        Ok(Cursor::new(cursor))
    }
}
