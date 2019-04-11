use bson;
use bson::Bson;
use failure::ResultExt;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::UpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;

use super::super::super::Cursor;
use super::super::super::ErrorKind;
use super::super::super::Result;

use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


/// Subset of the `Store` trait that deals with agents.
pub struct AgentStore {
    client: Client,
    db: String,
}

impl AgentStore {
    pub fn new(client: Client, db: String) -> AgentStore {
        AgentStore { client, db }
    }

    pub fn agent(&self, cluster_id: String, host: String) -> Result<Option<Agent>> {
        let filter = doc!{
            "cluster_id" => cluster_id,
            "host" => host,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_agents();
        let agent = collection
            .find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if agent.is_none() {
            return Ok(None);
        }
        let agent = agent.unwrap();
        let agent = bson::from_bson::<Agent>(bson::Bson::Document(agent))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
        Ok(Some(agent))
    }

    pub fn agent_info(&self, cluster_id: String, host: String) -> Result<Option<AgentInfo>> {
        let filter = doc!{
            "cluster_id" => cluster_id,
            "host" => host,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_agents_info();
        let agent_info = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("findOne"))?;
        timer.observe_duration();
        if agent_info.is_none() {
            return Ok(None);
        }
        let agent_info = agent_info.unwrap();
        let agent_info = bson::from_bson::<AgentInfo>(bson::Bson::Document(agent_info))
            .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
        Ok(Some(agent_info))
    }

    pub fn cluster_agents(&self, cluster_id: String) -> Result<Cursor<Agent>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_agents();
        let cursor = collection.find(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        timer.observe_duration();
        let iter = cursor.map(|doc| {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let agent = bson::from_bson::<Agent>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            Ok(agent.into())
        });
        Ok(Cursor(Box::new(iter)))
    }

    pub fn cluster_agents_info(&self, cluster_id: String) -> Result<Cursor<AgentInfo>> {
        let filter = doc!{"cluster_id" => cluster_id};
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_agents_info();
        let cursor = collection.find(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("find"))?;
        timer.observe_duration();
        let iter = cursor.map(|doc| {
            let doc = doc.with_context(|_| ErrorKind::MongoDBCursor("find"))?;
            let agent = bson::from_bson::<AgentInfo>(bson::Bson::Document(doc))
                .with_context(|_| ErrorKind::MongoDBBsonDecode)?;
            Ok(agent.into())
        });
        Ok(Cursor(Box::new(iter)))
    }

    pub fn persist_agent(&self, agent: Agent) -> Result<()> {
        let replacement = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("Agent failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster_id" => agent.cluster_id,
            "host" => agent.host,
        };
        let collection = self.collection_agents();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        Ok(())
    }

    pub fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        let replacement = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("AgentInfo failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster_id" => agent.cluster_id,
            "host" => agent.host,
        };
        let collection = self.collection_agents_info();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .with_context(|_| ErrorKind::MongoDBOperation("replaceOne"))?;
        Ok(())
    }

    /// Returns the `agents` collection.
    fn collection_agents(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_AGENTS)
    }

    /// Returns the `agents_info` collection.
    fn collection_agents_info(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO)
    }
}
