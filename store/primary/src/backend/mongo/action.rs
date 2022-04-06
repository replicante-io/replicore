use std::sync::Arc;

use bson::doc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::actions::Action as ActionModel;

use super::constants::COLLECTION_ACTIONS;
use crate::backend::ActionInterface;
use crate::store::action::ActionAttributes;
use crate::ErrorKind;
use crate::Result;

/// Action operations implementation using MongoDB.
pub struct Action {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Action {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Action
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Action { client, db, tracer }
    }
}

impl ActionInterface for Action {
    fn get(
        &self,
        attrs: &ActionAttributes,
        span: Option<SpanContext>,
    ) -> Result<Option<ActionModel>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "action_id": attrs.action_id.to_string(),
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let action = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(action)
    }
}
