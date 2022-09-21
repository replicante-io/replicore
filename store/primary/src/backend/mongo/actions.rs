use std::sync::Arc;

use bson::doc;
use failure::Fail;
use failure::ResultExt;
use mongodb::options::FindOptions;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_with_options;
use replicante_models_core::actions::node::ActionSyncSummary;
use replicante_models_core::api::node_action::NodeActionSummary;

use super::constants::COLLECTION_ACTIONS;
use super::document::NodeActionSummaryDocument;
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
    fn iter_summary(
        &self,
        attrs: &ActionsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<NodeActionSummary>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
        };
        let mut options = FindOptions::default();
        options.projection = Some(doc! {
            "cluster_id": 1,
            "node_id": 1,
            "action_id": 1,
            "created_ts": 1,
            "finished_ts": 1,
            "kind": 1,
            "scheduled_ts": 1,
            "state": 1,
        });
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let cursor = find_with_options(collection, filter, options, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let cursor = cursor.map(|document| {
            let document: NodeActionSummaryDocument =
                document.with_context(|_| ErrorKind::MongoDBCursor)?;
            Ok(NodeActionSummary::from(document))
        });
        Ok(Cursor::new(cursor))
    }

    fn unfinished_summaries(
        &self,
        attrs: &ActionsAttributes,
        span: Option<SpanContext>,
    ) -> Result<Cursor<ActionSyncSummary>> {
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
