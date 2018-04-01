use std::collections::HashMap;
use std::sync::Mutex;

use replicante_data_models::Node;

use super::InnerStore;
use super::Result;


/// A mock implementation of the storage layer for tests.
pub struct MockStore {
    pub nodes: Mutex<HashMap<(String, String), Node>>,
}

impl InnerStore for MockStore {
    fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        let cluster = node.info.datastore.cluster.clone();
        let name = node.info.datastore.name.clone();
        let key = (cluster, name);
        let mut nodes = self.nodes.lock().unwrap();
        let old = nodes.get(&key).map(|n| n.clone());
        nodes.insert(key, node);
        Ok(old)
    }
}

impl MockStore {
    /// Creates a new, empty, mock store.
    pub fn new() -> MockStore {
        MockStore {
            nodes: Mutex::new(HashMap::new()),
        }
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::AgentVersion;
    use replicante_agent_models::DatastoreInfo;
    use replicante_agent_models::NodeInfo;
    use replicante_agent_models::NodeStatus;
    use replicante_agent_models::Shard;
    use replicante_agent_models::ShardRole;
    use replicante_data_models::Node;

    fn make_node() -> Node {
        let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
        let datastore = DatastoreInfo::new("Cluster", "MockDB", "node", "1.2.3");
        let info = NodeInfo::new(agent, datastore);
        let status = NodeStatus::new(vec![Shard::new("id", ShardRole::Secondary, Some(4), 5)]);
        Node::new(info, status)
    }

    mod node {
        use std::sync::Arc;

        use super::super::super::Store;
        use super::super::MockStore;
        use super::make_node;

        #[test]
        fn persist_new() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node = make_node();
            let old = store.persist_node(node.clone()).unwrap();
            assert!(old.is_none());
            let stored = mock.nodes.lock().expect("Faild to lock")
                .get(&(node.info.datastore.cluster.clone(), node.info.datastore.name.clone()))
                .map(|n| n.clone()).expect("Node not found");
            assert_eq!(node, stored)
        }

        #[test]
        fn persist_replace() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node1 = make_node();
            let node2 = {
                let mut node = make_node();
                node.info.datastore.version = String::from("2.0.0");
                node
            };
            store.persist_node(node1.clone()).unwrap();
            let old = store.persist_node(node2).unwrap();
            assert_eq!(Some(node1), old);
        }
    }
}
