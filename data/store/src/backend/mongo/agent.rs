use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOneAndUpdateOptions;
use mongodb::db::ThreadedDatabase;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;

use super::constants::FAIL_FIND_AGENT;
use super::constants::FAIL_FIND_AGENT_INFO;
use super::constants::FAIL_PERSIST_AGENT;
use super::constants::FAIL_PERSIST_AGENT_INFO;

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

    pub fn agent(&self, cluster: String, host: String) -> Result<Option<Agent>> {
        let filter = doc!{
            "cluster" => cluster,
            "host" => host,
        };
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_agents();
        let agent = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_AGENT)?;
        timer.observe_duration();
        if agent.is_none() {
            return Ok(None);
        }
        let agent = agent.unwrap();
        let agent = bson::from_bson::<Agent>(bson::Bson::Document(agent))
            .chain_err(|| FAIL_FIND_AGENT)?;
        Ok(Some(agent))
    }

    pub fn agent_info(&self, cluster: String, host: String) -> Result<Option<AgentInfo>> {
        let filter = doc!{
            "cluster" => cluster,
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
            .chain_err(|| FAIL_FIND_AGENT_INFO)?;
        timer.observe_duration();
        if agent_info.is_none() {
            return Ok(None);
        }
        let agent_info = agent_info.unwrap();
        let agent_info = bson::from_bson::<AgentInfo>(bson::Bson::Document(agent_info))
            .chain_err(|| FAIL_FIND_AGENT_INFO)?;
        Ok(Some(agent_info))
    }

    pub fn persist_agent(&self, agent: Agent) -> Result<Option<Agent>> {
        let replacement = bson::to_bson(&agent).chain_err(|| FAIL_PERSIST_AGENT)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterMeta failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster" => agent.cluster,
            "host" => agent.host,
        };
        let collection = self.collection_agents();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_AGENT)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let agent = bson::from_bson::<Agent>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_AGENT)?;
                Ok(Some(agent))
            }
        }
    }

    pub fn persist_agent_info(&self, agent: AgentInfo) -> Result<Option<AgentInfo>> {
        let replacement = bson::to_bson(&agent).chain_err(|| FAIL_PERSIST_AGENT_INFO)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterMeta failed to encode as BSON document")
        };
        let filter = doc!{
            "cluster" => agent.cluster,
            "host" => agent.host,
        };
        let collection = self.collection_agents_info();
        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["findOneAndReplace"]).start_timer();
        let old = collection.find_one_and_replace(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOneAndReplace"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_AGENT_INFO)?;
        match old {
            None => Ok(None),
            Some(doc) => {
                let agent = bson::from_bson::<AgentInfo>(bson::Bson::Document(doc))
                    .chain_err(|| FAIL_PERSIST_AGENT_INFO)?;
                Ok(Some(agent))
            }
        }
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
