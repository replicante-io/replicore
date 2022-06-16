use std::collections::HashSet;

use lazy_static::lazy_static;

pub const COLLECTION_ACTIONS: &str = "actions";
pub const COLLECTION_ACTIONS_ORCHESTRATOR: &str = "actions_orchestrator";
pub const COLLECTION_AGENTS: &str = "agents";
pub const COLLECTION_AGENTS_INFO: &str = "agents_info";
pub const COLLECTION_CLUSTER_META: &str = "clusters_meta";
pub const COLLECTION_CLUSTER_SETTINGS: &str = "cluster_settings";
pub const COLLECTION_DISCOVERIES: &str = "discoveries";
pub const COLLECTION_DISCOVERY_SETTINGS: &str = "discovery_settings";
pub const COLLECTION_NODES: &str = "nodes";
pub const COLLECTION_SHARDS: &str = "shards";

pub const TOP_CLUSTERS_LIMIT: u32 = 10;

lazy_static! {
    pub static ref REMOVED_COLLECTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("events");
        set
    };
    pub static ref VALIDATE_EXPECTED_COLLECTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert(COLLECTION_ACTIONS);
        set.insert(COLLECTION_ACTIONS_ORCHESTRATOR);
        set.insert(COLLECTION_AGENTS);
        set.insert(COLLECTION_AGENTS_INFO);
        set.insert(COLLECTION_CLUSTER_META);
        set.insert(COLLECTION_CLUSTER_SETTINGS);
        set.insert(COLLECTION_DISCOVERIES);
        set.insert(COLLECTION_DISCOVERY_SETTINGS);
        set.insert(COLLECTION_NODES);
        set.insert(COLLECTION_SHARDS);
        set
    };
}
