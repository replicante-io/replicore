use std::sync::Arc;

use bson::doc;
use bson::Bson;
use bson::DateTime as UtcDateTime;
use chrono::DateTime;
use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find_one;
use replicante_externals_mongodb::operations::find_with_options;
use replicante_externals_mongodb::operations::update_many;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;

use super::super::ActionsInterface;
use super::constants::COLLECTION_ACTIONS;
use super::constants::COLLECTION_ACTIONS_HISTORY;
use super::constants::MAX_ACTIONS_SEARCH;
use super::document::ActionDocument;
use super::document::ActionHistoryDocument;
use crate::store::actions::SearchFilters;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Actions operations implementation using MongoDB.
pub struct Actions {
    client: Client,
    cluster_id: String,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Actions {
    pub fn new<T>(client: Client, db: String, tracer: T, cluster_id: String) -> Actions
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Actions {
            client,
            cluster_id,
            db,
            tracer,
        }
    }
}

impl ActionsInterface for Actions {
    fn action(&self, action_id: Uuid, span: Option<SpanContext>) -> Result<Option<Action>> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let filter = doc! {
            "cluster_id": &self.cluster_id,
            "action_id": action_id.to_string(),
        };
        let action: Option<ActionDocument> =
            find_one(collection, filter, span, self.tracer.as_deref())
                .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(action.map(Action::from))
    }

    fn finish_history(
        &self,
        action_id: Uuid,
        finished_ts: DateTime<Utc>,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "cluster_id": &self.cluster_id,
            "action_id": action_id.to_string(),
        };
        let finished_ts = UtcDateTime::from(finished_ts);
        let update = doc! {
            "$set": {
                "finished_ts": bson::to_bson(&finished_ts).unwrap(),
            }
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_HISTORY);
        update_many(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn history(&self, action_id: Uuid, span: Option<SpanContext>) -> Result<Vec<ActionHistory>> {
        // Prepare options and filters.
        let mut options = FindOptions::default();
        options.sort = Some(doc! {"timestamp": -1});
        let filters = doc! {
            "$and": [
                {"cluster_id": {"$eq": &self.cluster_id}},
                {"action_id": {"$eq": action_id.to_string()}},
            ],
        };

        // Execute the query.
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_HISTORY);
        let cursor = find_with_options(collection, filters, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|item: Result<ActionHistoryDocument>| item.map(ActionHistory::from));
        let mut history = Vec::new();
        for item in cursor {
            history.push(item?);
        }
        Ok(history)
    }

    fn search(&self, search: SearchFilters, span: Option<SpanContext>) -> Result<Cursor<Action>> {
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
        if let Some(node_id) = search.node_id {
            let node_id = regex::escape(&node_id);
            filters.push(Bson::from(doc! {"node_id": {"$regex": node_id}}));
        }
        let filters = doc! {"$and": filters};

        // Execute the query.
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find_with_options(collection, filters, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|item: Result<ActionDocument>| item.map(Action::from));
        Ok(Cursor::new(cursor))
    }
}
