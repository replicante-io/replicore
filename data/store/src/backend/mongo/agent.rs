use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Agent as AgentModel;
use replicante_data_models::AgentInfo as AgentInfoModel;

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
}

impl Agent {
    pub fn new(client: Client, db: String) -> Agent {
        Agent { client, db }
    }
}

impl AgentInterface for Agent {
    fn get(&self, attrs: &AgentAttribures) -> Result<Option<AgentModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "host" => &attrs.host,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        find_one(collection, filter)
    }

    fn info(&self, attrs: &AgentAttribures) -> Result<Option<AgentInfoModel>> {
        let filter = doc! {
            "cluster_id" => &attrs.cluster_id,
            "host" => &attrs.host,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        let document: Option<AgentInfoDocument> = find_one(collection, filter)?;
        Ok(document.map(AgentInfoModel::from))
    }
}
