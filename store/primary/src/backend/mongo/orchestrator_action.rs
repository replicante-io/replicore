use std::sync::Arc;

use bson::doc;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::actions::orchestrator::OrchestratorAction as OrchestratorActionModel;

use super::constants::COLLECTION_ACTIONS_ORCHESTRATOR;
use crate::backend::OrchestratorActionInterface;
use crate::store::orchestrator_action::OrchestratorActionAttributes;
use crate::ErrorKind;
use crate::Result;

/// OrchestratorAction operations implementation using MongoDB.
pub struct OrchestratorAction {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl OrchestratorAction {
    pub fn new<T>(client: Client, db: String, tracer: T) -> OrchestratorAction
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        OrchestratorAction { client, db, tracer }
    }
}

impl OrchestratorActionInterface for OrchestratorAction {
    fn get(
        &self,
        attrs: &OrchestratorActionAttributes,
        span: Option<SpanContext>,
    ) -> Result<Option<OrchestratorActionModel>> {
        let filter = doc! {
            "cluster_id": &attrs.cluster_id,
            "action_id": attrs.action_id.to_string(),
        };
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let action = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(action)
    }
}
