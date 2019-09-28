use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::Bson;
use bson::Document;
use bson::UtcDateTime;
use chrono::Utc;
use failure::ResultExt;
use mongodb::coll::options::FindOptions;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_externals_mongodb::operations::update_many;
use replicante_models_core::actions::ActionState;

use super::constants::COLLECTION_ACTIONS;
use crate::backend::ActionsInterface;
use crate::store::actions::ActionSyncState;
use crate::store::actions::ActionsAttributes;
use crate::ErrorKind;
use crate::Result;

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
    fn mark_lost(
        &self,
        attrs: &ActionsAttributes,
        node_id: String,
        refresh_id: i64,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "finished_ts": null,
            "node_id": node_id,
            "refresh_id": { "$ne": refresh_id }
        };
        let now = UtcDateTime(Utc::now());
        let update = doc! {
            "$set": {
                "finished_ts": bson::to_bson(&now).unwrap(),
                "state": bson::to_bson(&ActionState::Lost).unwrap(),
            }
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_ACTIONS);
        update_many(
            collection,
            filter,
            update,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(())
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
        let projection = doc! {
            "action_id" => 1,
            "finished_ts" => 1,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_ACTIONS);
        let mut options = FindOptions::default();
        options.projection = Some(projection);
        let cursor = find_with_options(
            collection,
            filter,
            options,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
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
            let uuid = Uuid::from_str(uuid).with_context(|_| ErrorKind::InvalidRecord(id))?;
            let finished_ts = if document.is_null("finished_ts") {
                ActionSyncState::Finished
            } else {
                ActionSyncState::Found
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
