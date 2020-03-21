use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::ordered::OrderedDocument;
use failure::Fail;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::aggregate;
use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Agent as AgentModel;
use replicante_models_core::agent::AgentInfo as AgentInfoModel;

use super::super::super::store::agents::AgentsAttribures;
use super::super::super::store::agents::AgentsCounts;
use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::AgentsInterface;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::document::AgentInfoDocument;

/// Return a document to count agents in given state as part of the $group stage.
fn aggregate_count_status(status: &'static str) -> OrderedDocument {
    doc! {"$sum" => {
        "$cond" => {
            "if" => {"$eq" => ["$status.code", status]},
            "then" => 1,
            "else" => 0,
        }
    }}
}

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
    fn counts(&self, attrs: &AgentsAttribures, span: Option<SpanContext>) -> Result<AgentsCounts> {
        // Let mongo figure out the counts with an aggregation.
        let filter = doc! {"$match" => {"cluster_id" => &attrs.cluster_id}};
        let agents_down = aggregate_count_status("AGENT_DOWN");
        let nodes = doc! {"$sum" => 1};
        let nodes_down = aggregate_count_status("NODE_DOWN");
        let group = doc! {"$group" => {
            "_id" => "$cluster_id",
            "agents_down" => agents_down,
            "nodes" => nodes,
            "nodes_down" => nodes_down,
        }};
        let pipeline = vec![filter, group];

        // Run aggrgation and grab the one and only (expected) result.
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        let mut cursor = aggregate(collection, pipeline, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let counts: AgentsCounts = match cursor.next() {
            Some(counts) => counts.with_context(|_| ErrorKind::MongoDBCursor)?,
            None => {
                return Ok(AgentsCounts {
                    agents_down: 0,
                    nodes: 0,
                    nodes_down: 0,
                })
            }
        };
        if cursor.next().is_some() {
            return Err(
                ErrorKind::DuplicateRecord("AgentsCounts", attrs.cluster_id.clone()).into(),
            );
        }
        Ok(counts)
    }

    fn iter(
        &self,
        attrs: &AgentsAttribures,
        span: Option<SpanContext>,
    ) -> Result<Cursor<AgentModel>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
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
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<AgentInfoDocument>| result.map(AgentInfoModel::from));
        Ok(Cursor::new(cursor))
    }
}
