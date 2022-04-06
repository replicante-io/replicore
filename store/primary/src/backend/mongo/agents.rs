use std::sync::Arc;

use bson::doc;
use failure::Fail;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;

use super::super::super::store::agents::AgentsAttribures;
use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::AgentsInterface;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;

/// Agents operations implementation using MongoDB.
pub struct Agents {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Agents {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Agents
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Agents { client, db, tracer }
    }
}

impl AgentsInterface for Agents {
    fn iter(
        &self,
        attrs: &AgentsAttribures,
        span: Option<SpanContext>,
    ) -> Result<Cursor<AgentModel>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self.client.database(&self.db).collection(COLLECTION_AGENTS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }

    fn iter_info(
        &self,
        attrs: &AgentsAttribures,
        span: Option<SpanContext>,
    ) -> Result<Cursor<AgentInfoModel>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self
            .client
            .database(&self.db)
            .collection(COLLECTION_AGENTS_INFO);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }
}
