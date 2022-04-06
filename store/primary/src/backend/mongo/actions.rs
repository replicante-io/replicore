use std::sync::Arc;

use bson::doc;
use bson::DateTime as UtcDateTime;
use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_externals_mongodb::operations::update_one;
use replicante_models_core::actions::ActionState;
use replicante_models_core::actions::ActionSummary;

use super::constants::COLLECTION_ACTIONS;
use crate::backend::ActionsInterface;
use crate::store::actions::ActionsAttributes;
use crate::Cursor;
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
            "$set": {"state": "PENDING_SCHEDULE"}
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
        let finished_ts = UtcDateTime::from(Utc::now());
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

    fn unfinished_summaries(
        &self,
        attrs: &ActionsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<ActionSummary>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "finished_ts": null,
        };
        let mut options = FindOptions::default();
        options.projection = Some(doc! {
            "cluster_id": 1,
            "node_id": 1,
            "action_id": 1,
            "state": 1,
        });
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find_with_options(collection, filter, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
