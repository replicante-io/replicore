use std::sync::Arc;

use bson::doc;
use bson::Bson;
use bson::DateTime as UtcDateTime;
use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use mongodb::sync::Collection;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use uuid::Uuid;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_externals_mongodb::operations::update_one;
use replicante_models_core::actions::node::ActionState;
use replicante_models_core::actions::node::ActionSummary;

use super::constants::COLLECTION_ACTIONS;
use crate::backend::ActionsInterface;
use crate::store::actions::ActionsAttributes;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Approve a PENDING_APPROVE node or orchestrator action.
///
/// Returns true if the action record was updated.
pub fn action_generic_approve(
    cluster_id: &str,
    action_id: Uuid,
    collection: Collection,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<bool> {
    let filter = doc! {
        "cluster_id": cluster_id,
        "action_id": action_id.to_string(),
        "state": "PENDING_APPROVE",
    };
    let update = doc! {
        "$set": {"state": "PENDING_SCHEDULE"}
    };
    let result = update_one(collection, filter, update, span, tracer)
        .with_context(|_| ErrorKind::MongoDBOperation)?;
    Ok(result.modified_count != 0)
}

/// Disapprove (reject) a PENDING_APPROVE node or orchestrator action.
///
/// Returns true if the action record was updated.
pub fn action_generic_disapprove(
    cluster_id: &str,
    action_id: Uuid,
    state: Bson,
    collection: Collection,
    span: Option<SpanContext>,
    tracer: Option<&Tracer>,
) -> Result<bool> {
    let filter = doc! {
        "cluster_id": &cluster_id,
        "action_id": action_id.to_string(),
        "state": "PENDING_APPROVE",
    };
    let finished_ts = UtcDateTime::from(Utc::now());
    let update = doc! {
        "$set": {
            "finished_ts": bson::to_bson(&finished_ts).unwrap(),
            "state": state,
        }
    };
    let result = update_one(collection, filter, update, span, tracer)
        .with_context(|_| ErrorKind::MongoDBOperation)?;
    Ok(result.modified_count != 0)
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
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        action_generic_approve(
            &attrs.cluster_id,
            action_id,
            collection,
            span,
            self.tracer.as_deref(),
        )?;
        Ok(())
    }

    fn disapprove(
        &self,
        attrs: &ActionsAttributes,
        action_id: Uuid,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        action_generic_disapprove(
            &attrs.cluster_id,
            action_id,
            bson::to_bson(&ActionState::Cancelled).unwrap(),
            collection,
            span,
            self.tracer.as_deref(),
        )?;
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
