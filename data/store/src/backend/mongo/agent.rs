use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_models_core::Agent as AgentModel;
use replicante_models_core::AgentInfo as AgentInfoModel;

use super::super::super::store::agent::AgentAttribures;
use super::super::super::Result;
use super::super::AgentInterface;
use super::common::find_one;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::document::AgentInfoDocument;

/// Agent operations implementation using MongoDB.
pub struct Agent {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Agent {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Agent
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Agent { client, db, tracer }
    }
}

impl AgentInterface for Agent {
    fn get(
        &self,
        attrs: &AgentAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<AgentModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "host" => &attrs.host,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn info(
        &self,
        attrs: &AgentAttribures,
        span: Option<SpanContext>,
    ) -> Result<Option<AgentInfoModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "host" => &attrs.host,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        let document: Option<AgentInfoDocument> = find_one(
            collection,
            filter,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )?;
        Ok(document.map(AgentInfoModel::from))
    }
}
