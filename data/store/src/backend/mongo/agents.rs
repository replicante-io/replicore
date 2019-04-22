use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_data_models::Agent as AgentModel;
use replicante_data_models::AgentInfo as AgentInfoModel;

use super::super::super::store::agents::AgentsAttribures;
use super::super::super::Cursor;
use super::super::super::Result;
use super::super::AgentsInterface;
use super::common::find;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;

/// Agents operations implementation using MongoDB.
pub struct Agents {
    client: Client,
    db: String,
}

impl Agents {
    pub fn new(client: Client, db: String) -> Agents {
        Agents { client, db }
    }
}

impl AgentsInterface for Agents {
    fn iter(&self, attrs: &AgentsAttribures) -> Result<Cursor<AgentModel>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        find(collection, filter)
    }

    fn iter_info(&self, attrs: &AgentsAttribures) -> Result<Cursor<AgentInfoModel>> {
        let filter = doc! {"cluster_id" => &attrs.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        find(collection, filter)
    }
}
