use std::sync::Arc;

use bson::doc;
use bson::Bson;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::insert_many;
use replicante_externals_mongodb::operations::insert_one;
use replicante_externals_mongodb::operations::replace_one;
use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;
use replicante_models_core::actions::orchestrator::OrchestratorAction;
use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::events::Event;

use super::super::PersistInterface;
use super::constants::COLLECTION_ACTIONS;
use super::constants::COLLECTION_ACTIONS_HISTORY;
use super::constants::COLLECTION_ACTIONS_ORCHESTRATOR;
use super::constants::COLLECTION_CLUSTER_ORCHESTRATE_REPORT;
use super::constants::COLLECTION_EVENTS;
use super::document::ActionDocument;
use super::document::ActionHistoryDocument;
use super::document::EventDocument;
use super::document::OrchestrateReportDocument;
use super::document::OrchestratorActionDocument;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Persistence operations implementation using MongoDB.
pub struct Persist {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Persist {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Persist
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Persist { client, db, tracer }
    }
}

impl PersistInterface for Persist {
    fn action(&self, action: Action, span: Option<SpanContext>) -> Result<()> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS);
        let action = ActionDocument::from(action);
        let filter = doc! {
            "cluster_id": &action.cluster_id,
            "node_id": &action.node_id,
            "action_id": &action.action_id,
        };
        let action = bson::to_bson(&action).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let action = match action {
            Bson::Document(action) => action,
            _ => panic!("Action failed to encode as BSON document"),
        };
        replace_one(collection, filter, action, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)
            .map_err(Error::from)?;
        Ok(())
    }

    fn action_history(&self, history: Vec<ActionHistory>, span: Option<SpanContext>) -> Result<()> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_HISTORY);
        let mut records = Vec::new();
        for item in history.into_iter() {
            let item = ActionHistoryDocument::from(item);
            let document = bson::to_bson(&item).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
            let document = match document {
                Bson::Document(document) => document,
                _ => panic!("Event failed to encode as BSON document"),
            };
            records.push(document);
        }
        insert_many(collection, records, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)
            .map_err(Error::from)
    }

    fn cluster_orchestrate_report(
        &self,
        report: OrchestrateReport,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_CLUSTER_ORCHESTRATE_REPORT);
        let report = OrchestrateReportDocument::from(report);
        let filter = doc! {
            "namespace": &report.namespace,
            "cluster_id": &report.cluster_id,
        };
        let report = bson::to_bson(&report).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let report = match report {
            Bson::Document(report) => report,
            _ => panic!("OrchestrateReport failed to encode as BSON document"),
        };
        replace_one(collection, filter, report, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)
            .map_err(Error::from)?;
        Ok(())
    }

    fn event(&self, event: Event, span: Option<SpanContext>) -> Result<()> {
        let collection = self.client.database(&self.db).collection(COLLECTION_EVENTS);
        let event = EventDocument::from(event);
        let document = bson::to_bson(&event).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Event failed to encode as BSON document"),
        };
        insert_one(collection, document, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)
            .map_err(Error::from)
    }

    fn orchestrator_action(
        &self,
        action: OrchestratorAction,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_ACTIONS_ORCHESTRATOR);
        let action = OrchestratorActionDocument::from(action);
        let filter = doc! {
            "cluster_id": &action.cluster_id,
            "action_id": &action.action_id,
        };
        let action = bson::to_bson(&action).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let action = match action {
            Bson::Document(action) => action,
            _ => panic!("OrchestratorAction failed to encode as BSON document"),
        };
        replace_one(collection, filter, action, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)
            .map_err(Error::from)?;
        Ok(())
    }
}
