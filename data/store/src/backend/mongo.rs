use replicante_data_models::Node;

use super::super::InnerStore;
use super::super::Result;
use super::super::config::MongoDBConfig;


pub struct MongoStore {
    // TODO
}

impl InnerStore for MongoStore {
    // TODO: update this method once the agent returns the cluster ID.
    fn persist_node(&self, _node: Node) -> Result<Option<Node>> {
        Err("TODO".into())
    }
}

impl MongoStore {
    pub fn new(_config: MongoDBConfig) -> Result<MongoStore> {
        Ok(MongoStore {})
    }
}
