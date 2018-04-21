use std::collections::HashMap;
use std::sync::Mutex;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;

use replicante_data_models::Node;
use replicante_data_models::Shard;

use replicante_data_models::Event;

use super::InnerStore;
use super::Result;


/// A mock implementation of the storage layer for tests.
pub struct MockStore {
    pub agents: Mutex<HashMap<(String, String), Agent>>,
    pub agents_info: Mutex<HashMap<(String, String), AgentInfo>>,
    pub clusters_meta: Mutex<HashMap<String, ClusterMeta>>,
    pub discoveries: Mutex<HashMap<String, ClusterDiscovery>>,
    pub nodes: Mutex<HashMap<(String, String), Node>>,
    pub shards: Mutex<HashMap<(String, String, String), Shard>>,
    pub events: Mutex<Vec<Event>>,
}

impl InnerStore for MockStore {
    fn cluster_discovery(&self, cluster: String) -> Result<ClusterDiscovery> {
        let discoveries = self.discoveries.lock().unwrap();
        let discovery: Result<ClusterDiscovery> = discoveries.get(&cluster).map(|c| c.clone())
            .ok_or("Cluster not found".into());
        Ok(discovery?)
    }

    fn cluster_meta(&self, cluster: String) -> Result<ClusterMeta> {
        let clusters = self.clusters_meta.lock().unwrap();
        let meta: Result<ClusterMeta> = clusters.get(&cluster).map(|c| c.clone())
            .ok_or("Cluster not found".into());
        Ok(meta?)
    }

    fn find_clusters(&self, search: String, _: u8) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let results = clusters.iter()
            .filter(|&(name, _)| name.contains(&search))
            .map(|(_, meta)| meta.clone())
            .collect();
        Ok(results)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let mut results: Vec<ClusterMeta> = clusters.iter()
            .map(|(_, meta)| meta.clone())
            .collect();
        results.sort_by_key(|meta| meta.nodes);
        Ok(results)
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<Option<ClusterDiscovery>> {
        let name = cluster.name.clone();
        let mut discoveries = self.discoveries.lock().unwrap();
        let old = discoveries.get(&name).map(|c| c.clone());
        discoveries.insert(name, cluster);
        Ok(old)
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<Option<ClusterMeta>> {
        let name = meta.name.clone();
        let mut clusters = self.clusters_meta.lock().unwrap();
        let old = clusters.get(&name).map(|m| m.clone());
        clusters.insert(name, meta);
        Ok(old)
    }

    fn persist_node(&self, node: Node) -> Result<Option<Node>> {
        let cluster = node.cluster.clone();
        let name = node.name.clone();
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
            agents: Mutex::new(HashMap::new()),
            agents_info: Mutex::new(HashMap::new()),
            clusters_meta: Mutex::new(HashMap::new()),
            discoveries: Mutex::new(HashMap::new()),
            nodes: Mutex::new(HashMap::new()),
            shards: Mutex::new(HashMap::new()),
            events: Mutex::new(Vec::new()),
        }
    }
}


#[cfg(test)]
mod tests {
    mod cluster_discovery {
        use std::sync::Arc;
        use replicante_data_models::ClusterDiscovery;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn found_discovery() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            mock.discoveries.lock().expect("Faild to lock").insert("test".into(), cluster.clone());
            let found = store.cluster_discovery("test").unwrap();
            assert_eq!(found, cluster);
        }

        #[test]
        fn missing_discovery() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            match store.cluster_discovery("test") {
                Ok(_) => panic!("Unexpected cluster found"),
                Err(_) => ()
            }
        }

        #[test]
        fn persist_new() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let old = store.persist_discovery(cluster.clone()).unwrap();
            assert!(old.is_none());
            let stored = mock.discoveries.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(cluster, stored)
        }

        #[test]
        fn persist_update() {
            let discovery1 = ClusterDiscovery::new("test", vec!["test1".into()]);
            let discovery2 = ClusterDiscovery::new("test", vec!["test2".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            store.persist_discovery(discovery1.clone()).unwrap();
            let old = store.persist_discovery(discovery2).unwrap();
            assert_eq!(Some(discovery1), old);
        }
    }

    mod cluster_meta {
        use std::sync::Arc;
        use replicante_data_models::ClusterMeta;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn find_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster1 = ClusterMeta::new("cluster1", "Redis", 44);
            let cluster2 = ClusterMeta::new("cluster2", "Redis", 44);
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster1".into(), cluster1.clone());
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster2".into(), cluster2.clone());
            let results = store.find_clusters("2", 33).unwrap();
            assert_eq!(results, vec![cluster2]);
        }

        #[test]
        fn found_meta() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let meta = ClusterMeta::new("test", "Redis", 44);
            mock.clusters_meta.lock().expect("Faild to lock").insert("test".into(), meta.clone());
            let found = store.cluster_meta("test").unwrap();
            assert_eq!(found, meta);
        }

        #[test]
        fn missing_meta() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            match store.cluster_meta("test") {
                Ok(_) => panic!("Unexpected cluster found"),
                Err(_) => ()
            }
        }

        #[test]
        fn top_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster1 = ClusterMeta::new("cluster1", "Redis", 44);
            let cluster2 = ClusterMeta::new("cluster2", "Redis", 4);
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster1".into(), cluster1.clone());
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster2".into(), cluster2.clone());
            let results = store.top_clusters().unwrap();
            assert_eq!(results, vec![cluster2, cluster1]);
        }

        #[test]
        fn persist_new() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let meta = ClusterMeta::new("test", "Redis", 44);
            let old = store.persist_cluster_meta(meta.clone()).unwrap();
            assert!(old.is_none());
            let stored = mock.clusters_meta.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(meta, stored)
        }

        #[test]
        fn persist_update() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let meta1 = ClusterMeta::new("test", "Redis", 4);
            let meta2 = ClusterMeta::new("test", "Redis", 44);
            store.persist_cluster_meta(meta1.clone()).unwrap();
            let old = store.persist_cluster_meta(meta2).unwrap();
            assert_eq!(Some(meta1), old);
        }
    }

    mod node {
        use std::sync::Arc;
        use replicante_agent_models::DatastoreInfo;
        use replicante_data_models::Node;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist_new() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node = Node::new(DatastoreInfo::new("cluster", "kind", "name", "version"));
            let old = store.persist_node(node.clone()).unwrap();
            assert!(old.is_none());
            let key = (String::from("cluster"), String::from("name"));
            let stored = mock.nodes.lock().expect("Faild to lock")
                .get(&key)
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(node, stored)
        }

        #[test]
        fn persist_update() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node1 = Node::new(DatastoreInfo::new("cluster", "kind1", "name", "version"));
            let node2 = Node::new(DatastoreInfo::new("cluster", "kind2", "name", "version"));
            store.persist_node(node1.clone()).unwrap();
            let old = store.persist_node(node2).unwrap();
            assert_eq!(Some(node1), old);
        }
    }
}
