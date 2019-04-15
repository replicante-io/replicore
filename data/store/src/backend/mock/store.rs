use std::collections::HashMap;
use std::sync::Mutex;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::super::Cursor;
use super::super::super::EventsFilters;
use super::super::super::EventsOptions;

use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::super::store::InnerStore;


/// A mock implementation of the storage layer for tests.
#[derive(Default)]
pub struct MockStore {
    pub agents: Mutex<HashMap<(String, String), Agent>>,
    pub agents_info: Mutex<HashMap<(String, String), AgentInfo>>,
    pub clusters_meta: Mutex<HashMap<String, ClusterMeta>>,
    pub discoveries: Mutex<HashMap<String, ClusterDiscovery>>,
    pub events: Mutex<Vec<Event>>,
    pub nodes: Mutex<HashMap<(String, String), Node>>,
    pub shards: Mutex<HashMap<(String, String, String), Shard>>,
}

impl InnerStore for MockStore {
    fn agent(&self, cluster_id: String, host: String) -> Result<Option<Agent>> {
        let agents = self.agents.lock().unwrap();
        let agent = agents.get(&(cluster_id, host)).cloned();
        Ok(agent)
    }

    fn agent_info(&self, cluster_id: String, host: String) -> Result<Option<AgentInfo>> {
        let agents_info = self.agents_info.lock().unwrap();
        let agent_info = agents_info.get(&(cluster_id, host)).cloned();
        Ok(agent_info)
    }

    fn cluster_agents(&self, _cluster_id: String) -> Result<Cursor<Agent>> {
        Err(ErrorKind::MockNotYetImplemented("cluster agents").into())
    }

    fn cluster_agents_info(&self, _cluster_id: String) -> Result<Cursor<AgentInfo>> {
        Err(ErrorKind::MockNotYetImplemented("cluster agents info").into())
    }

    fn cluster_discovery(&self, cluster_id: String) -> Result<Option<ClusterDiscovery>> {
        let discoveries = self.discoveries.lock().unwrap();
        let discovery = discoveries.get(&cluster_id).cloned();
        Ok(discovery)
    }

    fn cluster_meta(&self, cluster_id: String) -> Result<Option<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let meta = clusters.get(&cluster_id).cloned();
        Ok(meta)
    }

    fn cluster_nodes(&self, _cluster_id: String) -> Result<Cursor<Node>> {
        Err(ErrorKind::MockNotYetImplemented("cluster nodes").into())
    }

    fn cluster_shards(&self, _cluster_id: String) -> Result<Cursor<Shard>> {
        Err(ErrorKind::MockNotYetImplemented("cluster shards").into())
    }

    fn events(&self, _filters: EventsFilters, _options: EventsOptions) -> Result<Cursor<Event>> {
        let events = self.events.lock().unwrap().clone();
        let events: Vec<_> = events.into_iter().rev().collect();
        let iter = events.into_iter().map(|e| Ok(e));
        Ok(Cursor(Box::new(iter)))
    }

    fn find_clusters(&self, search: String, _: u8) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let results = clusters.iter()
            .filter(|&(name, _)| name.contains(&search))
            .map(|(_, meta)| meta.clone())
            .collect();
        Ok(results)
    }
    
    fn node(&self, cluster_id: String, name: String) -> Result<Option<Node>> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&(cluster_id, name)).cloned();
        Ok(node)
    }

    fn persist_agent(&self, agent: Agent) -> Result<()> {
        let cluster_id = agent.cluster_id.clone();
        let host = agent.host.clone();
        let mut agents = self.agents.lock().unwrap();
        agents.insert((cluster_id, host), agent);
        Ok(())
    }

    fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        let cluster_id = agent.cluster_id.clone();
        let host = agent.host.clone();
        let mut agents_info = self.agents_info.lock().unwrap();
        agents_info.insert((cluster_id, host), agent);
        Ok(())
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        let cluster_id = cluster.cluster_id.clone();
        let mut discoveries = self.discoveries.lock().unwrap();
        discoveries.insert(cluster_id, cluster);
        Ok(())
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        let cluster_id = meta.cluster_id.clone();
        let mut clusters = self.clusters_meta.lock().unwrap();
        clusters.insert(cluster_id, meta);
        Ok(())
    }

    fn persist_event(&self, event: Event) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event);
        Ok(())
    }

    fn persist_node(&self, node: Node) -> Result<()> {
        let cluster_id = node.cluster_id.clone();
        let node_id = node.node_id.clone();
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert((cluster_id, node_id), node);
        Ok(())
    }

    fn persist_shard(&self, shard: Shard) -> Result<()> {
        let cluster_id = shard.cluster_id.clone();
        let node_id = shard.node_id.clone();
        let shard_id = shard.shard_id.clone();
        let mut shards = self.shards.lock().unwrap();
        shards.insert((cluster_id, node_id, shard_id), shard);
        Ok(())
    }

    fn shard(&self, cluster_id: String, node: String, id: String) -> Result<Option<Shard>> {
        let shards = self.shards.lock().unwrap();
        let shard = shards.get(&(cluster_id, node, id)).cloned();
        Ok(shard)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let mut results: Vec<ClusterMeta> = clusters.iter()
            .map(|(_, meta)| meta.clone())
            .collect();
        results.sort_by_key(|meta| meta.nodes);
        Ok(results)
    }
}

impl MockStore {
    /// Creates a new, empty, mock store.
    pub fn new() -> MockStore {
        MockStore::default()
    }
}

#[cfg(test)]
mod tests {
    mod agent {
        use std::sync::Arc;
        use replicante_data_models::Agent;
        use replicante_data_models::AgentStatus;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn get() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let agent = Agent::new("test", "node", AgentStatus::Up);
            let key = (String::from("test"), String::from("node"));
            mock.agents.lock().unwrap().insert(key.clone(), agent.clone());
            let stored = store.agent(key.0, key.1).unwrap().unwrap();
            assert_eq!(stored, agent);
        }

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let agent = Agent::new("test", "node", AgentStatus::Up);
            store.persist_agent(agent.clone()).unwrap();
            let stored = mock.agents.lock().expect("Faild to lock")
                .get(&("test".into(), "node".into()))
                .map(|n| n.clone()).expect("Agent not found");
            assert_eq!(agent, stored)
        }
    }

    mod agent_info {
        use std::sync::Arc;
        use replicante_data_models::AgentInfo;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let info = AgentInfo {
                cluster_id: "test".into(),
                host: "node".into(),
                version_checkout: "commit".into(),
                version_number: "1.2.3".into(),
                version_taint: "yep".into(),
            };
            store.persist_agent_info(info.clone()).unwrap();
            let stored = mock.agents_info.lock().expect("Faild to lock")
                .get(&("test".into(), "node".into()))
                .map(|n| n.clone()).expect("Agent not found");
            assert_eq!(info, stored);
        }
    }

    mod cluster_meta {
        use std::sync::Arc;
        use replicante_data_models::ClusterMeta;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn find_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let mut cluster1 = ClusterMeta::new("cluster1", "Redis");
            cluster1.nodes = 44;
            let mut cluster2 = ClusterMeta::new("cluster2", "Redis");
            cluster2.nodes = 44;
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
            let mut meta = ClusterMeta::new("test", "Redis");
            meta.nodes = 44;
            mock.clusters_meta.lock().expect("Faild to lock").insert("test".into(), meta.clone());
            let found = store.cluster_meta("test").unwrap().unwrap();
            assert_eq!(found, meta);
        }

        #[test]
        fn missing_meta() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            assert!(store.cluster_meta("test").unwrap().is_none());
        }

        #[test]
        fn top_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let mut cluster1 = ClusterMeta::new("cluster1", "Redis");
            cluster1.nodes = 44;
            let mut cluster2 = ClusterMeta::new("cluster2", "Redis");
            cluster2.nodes = 4;
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster1".into(), cluster1.clone());
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster2".into(), cluster2.clone());
            let results = store.top_clusters().unwrap();
            assert_eq!(results, vec![cluster2, cluster1]);
        }

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let mut meta = ClusterMeta::new("test", "Redis");
            meta.nodes = 44;
            store.persist_cluster_meta(meta.clone()).unwrap();
            let stored = mock.clusters_meta.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(meta, stored);
        }
    }

    mod discovery {
        use std::sync::Arc;
        use replicante_data_models::ClusterDiscovery;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn found_discovery() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            mock.discoveries.lock().expect("Faild to lock").insert("test".into(), cluster.clone());
            let found = store.cluster_discovery("test").unwrap().unwrap();
            assert_eq!(found, cluster);
        }

        #[test]
        fn missing_discovery() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            assert!(store.cluster_discovery("test").unwrap().is_none());
        }

        #[test]
        fn persist() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            store.persist_discovery(cluster.clone()).unwrap();
            let stored = mock.discoveries.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(cluster, stored);
        }
    }

    mod event {
        use std::sync::Arc;
        use replicante_data_models::ClusterDiscovery;
        use replicante_data_models::Event;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let event = Event::builder().cluster().cluster_new(cluster);
            store.persist_event(event.clone()).unwrap();
            let stored = mock.events.lock().expect("Faild to lock").clone();
            assert_eq!(vec![event], stored);
        }
    }

    mod node {
        use std::sync::Arc;
        use replicante_agent_models::DatastoreInfo;
        use replicante_data_models::Node;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node = Node::new(DatastoreInfo::new("cluster", "kind", "name", "version", None));
            store.persist_node(node.clone()).unwrap();
            let key = (String::from("cluster"), String::from("name"));
            let stored = mock.nodes.lock().expect("Faild to lock")
                .get(&key)
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(node, stored);
        }
    }

    mod shards {
        use std::sync::Arc;
        use replicante_agent_models::Shard as WireShard;
        use replicante_data_models::CommitOffset;
        use replicante_data_models::Shard;
        use replicante_data_models::ShardRole;

        use super::super::super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let shard = Shard::new("cluster", "node", WireShard::new(
                "id", ShardRole::Primary, Some(CommitOffset::seconds(1)), None
            ));
            store.persist_shard(shard.clone()).unwrap();
            let key = (String::from("cluster"), String::from("node"), String::from("id"));
            let stored = mock.shards.lock().expect("Faild to lock")
                .get(&key)
                .map(|n| n.clone()).expect("Shard not found");
            assert_eq!(shard, stored)
        }
    }
}
