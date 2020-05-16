use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use bson::doc;
use bson::Bson;
use bson::Document;
use bson::UtcDateTime;
use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find;
use replicante_externals_mongodb::operations::update_many;
use replicante_externals_mongodb::operations::update_one;
use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionState;

use super::constants::COLLECTION_ACTIONS;
use super::document::ActionDocument;
use crate::backend::ActionsInterface;
use crate::store::actions::ActionSyncState;
use crate::store::actions::ActionsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Generate the lost actions filter document.
///
/// This is called by `Actions::iter_lost` and `Actions::mark_lost` to ensure
/// that both methods will always operate on the same set of documents.
fn lost_actions_filter(
    attrs: &ActionsAttributes,
    node_id: &str,
    refresh_id: i64,
) -> bson::Document {
    doc! {
        "cluster_id": &attrs.cluster_id,
        "finished_ts": null,
        "node_id": node_id,
        "refresh_id": { "$ne": refresh_id },
        "state": { "$nin": [
            "PENDING_APPROVE",
            "PENDING_SCHEDULE",
        ] }
    }
}

/// Actions operations implementation using MongoDB.
pub struct Actions {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Actions {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Actions
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Actions { client, db, tracer }
    }
}

impl ActionsInterface for Actions {
    fn approve(
        &self,
        attrs: &ActionsAttributes,
        action_id: Uuid,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "action_id": action_id.to_string(),
            "state": "PENDING_APPROVE",
        };
        let update = doc! {
            "$set": {
                "state": "PENDING_SCHEDULE",
            }
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        update_one(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn disapprove(
        &self,
        attrs: &ActionsAttributes,
        action_id: Uuid,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "action_id": action_id.to_string(),
            "state": "PENDING_APPROVE",
        };
        let finished_ts = UtcDateTime(Utc::now());
        let update = doc! {
            "$set": {
                "finished_ts": bson::to_bson(&finished_ts).unwrap(),
                "state": bson::to_bson(&ActionState::Cancelled).unwrap(),
            }
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        update_one(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn iter_lost(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        span: Option<SpanContext>,
    ) -> Result<Cursor<Action>> {
        let filter = lost_actions_filter(attrs, &node_id, refresh_id);
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        // Simulate the changes that will be performed by `mark_lost` for clients.
        let cursor = cursor.map(move |action| {
            let action: ActionDocument = action.with_context(|_| ErrorKind::MongoDBCursor)?;
            let mut action: Action = action.into();
            action.state = ActionState::Lost;
            action.finished_ts = Some(finished_ts);
            Ok(action)
        });
        Ok(Cursor::new(cursor))
    }

    fn mark_lost(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        refresh_id: i64,
        finished_ts: DateTime<Utc>,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = lost_actions_filter(attrs, &node_id, refresh_id);
        let finished_ts = UtcDateTime(finished_ts);
        let update = doc! {
            "$set": {
                "finished_ts": bson::to_bson(&finished_ts).unwrap(),
                "state": bson::to_bson(&ActionState::Lost).unwrap(),
            }
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        update_many(collection, filter, update, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
    }

    fn pending_schedule(
        &self,
        attrs: &ActionsAttributes,
        agent_id: String,
        span: Option<SpanContext>,
    ) -> Result<Cursor<Action>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &agent_id,
            "state" => bson::to_bson(&ActionState::PendingSchedule).unwrap(),
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|action| {
                let action: ActionDocument = action.with_context(|_| ErrorKind::MongoDBCursor)?;
                Ok(action.into())
            });
        Ok(Cursor::new(cursor))
    }

    fn state_for_sync(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        action_ids: &[Uuid],
        span: Option<SpanContext>,
    ) -> Result<HashMap<Uuid, ActionSyncState>> {
        let mut results = HashMap::new();

        // Check the actions in the DB first.
        let action_ids_bson: Vec<Bson> = action_ids
            .iter()
            .map(ToString::to_string)
            .map(Bson::from)
            .collect();
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "node_id" => &node_id,
            "action_id" => {
                "$in" => action_ids_bson,
            },
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        for document in cursor {
            let document: Document = document.with_context(|_| ErrorKind::MongoDBCursor)?;
            let id = document
                .get_object_id("_id")
                .map(bson::oid::ObjectId::to_hex)
                .unwrap_or_else(|_| "<NO ID>".into());
            let uuid = document
                .get_str("action_id")
                .with_context(|_| ErrorKind::InvalidRecord(id.clone()))?;
            let uuid =
                Uuid::from_str(uuid).with_context(|_| ErrorKind::InvalidRecord(id.clone()))?;
            let finished_ts = if document.is_null("finished_ts") {
                let action: ActionDocument = bson::from_bson(document.into())
                    .with_context(|_| ErrorKind::InvalidRecord(id))?;
                ActionSyncState::Found(action.into())
            } else {
                ActionSyncState::Finished
            };
            results.insert(uuid, finished_ts);
        }

        // Mark any action not in the results as NotFound.
        for action_id in action_ids {
            results
                .entry(*action_id)
                .or_insert(ActionSyncState::NotFound);
        }
        Ok(results)
    }
}
