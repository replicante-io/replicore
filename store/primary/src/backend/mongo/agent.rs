use std::sync::Arc;

use bson::bson;
use bson::doc;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find_one;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;

use super::super::AgentInterface;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::document::AgentInfoDocument;
use crate::store::agent::AgentAttribures;
use crate::ErrorKind;
use crate::Result;

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
        let agent = find_one(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(agent)
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
        let document: Option<AgentInfoDocument> =
            find_one(collection, filter, span, self.tracer.as_deref())
                .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(document.map(AgentInfoModel::from))
    }
}
