use std::collections::HashMap;
use std::sync::Mutex;

use replicante_data_models::Cluster;
use replicante_data_models::Node;

use replicante_data_models::webui::TopClusters;

use super::InnerStore;
use super::Result;


/// A mock implementation of the storage layer for tests.
pub struct MockStore {
    pub clusters: Mutex<HashMap<String, Cluster>>,
    pub nodes: Mutex<HashMap<(String, String), Node>>,
    pub top_clusters: TopClusters,
}

impl InnerStore for MockStore {
    fn find_clusters(&self, _: String, _: u8) -> Result<Vec<String>> {
        Ok(self.clusters.lock().unwrap().keys().map(|k| k.clone()).collect())
    }

    fn fetch_top_clusters(&self) -> Result<TopClusters> {
        Ok(self.top_clusters.clone())
    }

    fn persist_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>> {
        let name = cluster.name.clone();
        let mut clusters = self.clusters.lock().unwrap();
        let old = clusters.get(&name).map(|c| c.clone());
        clusters.insert(name, cluster);
        Ok(old)
    }

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
            clusters: Mutex::new(HashMap::new()),
            nodes: Mutex::new(HashMap::new()),
            top_clusters: Vec::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    mod cluster {
        use std::sync::Arc;
        use replicante_data_models::Cluster;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn find_clusters() {
            let cluster1 = Cluster::new("test1", vec!["test1".into()]);
            let cluster2 = Cluster::new("test2", vec!["test2".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            store.persist_cluster(cluster1).unwrap();
            store.persist_cluster(cluster2).unwrap();
            let mut clusters = store.find_clusters("ignored", 5).unwrap();
            clusters.sort();
            let expected: Vec<String> = vec!["test1".into(), "test2".into()];
            assert_eq!(clusters, expected);
        }

        #[test]
        fn persist_new() {
            let cluster = Cluster::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let old = store.persist_cluster(cluster.clone()).unwrap();
            assert!(old.is_none());
            let stored = mock.clusters.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(cluster, stored)
        }

        #[test]
        fn persist_update() {
            let cluster1 = Cluster::new("test", vec!["test1".into()]);
            let cluster2 = Cluster::new("test", vec!["test2".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            store.persist_cluster(cluster1.clone()).unwrap();
            let old = store.persist_cluster(cluster2).unwrap();
            assert_eq!(Some(cluster1), old);
        }
    }


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
