use std::collections::HashSet;


pub const COLLECTION_AGENTS: &str = "agents";
pub const COLLECTION_AGENTS_INFO: &str = "agents_info";
pub const COLLECTION_CLUSTER_META: &str = "clusters_meta";
pub const COLLECTION_DISCOVERIES: &str = "discoveries";
pub const COLLECTION_EVENTS: &str = "events";
pub const COLLECTION_NODES: &str = "nodes";
pub const COLLECTION_SHARDS: &str = "shards";


pub const FAIL_CLIENT: &str = "Failed to configure MongoDB client";

pub const FAIL_FIND_AGENT: &str = "Failed to find agent status";
pub const FAIL_FIND_AGENT_INFO: &str = "Failed to find agent info";
pub const FAIL_FIND_CLUSTER_DISCOVERY: &str = "Failed to find cluster discovery record";
pub const FAIL_FIND_CLUSTER_META: &str = "Failed to find cluster metadata";
pub const FAIL_FIND_CLUSTERS: &str = "Failed while searching for clusters";
pub const FAIL_FIND_NODE: &str = "Failed to find node information";
pub const FAIL_FIND_SHARD: &str = "Failed to find shard information";

pub const FAIL_PERSIST_AGENT: &str = "Failed to persist agent status";
pub const FAIL_PERSIST_AGENT_INFO: &str = "Failed to persist agent info";
pub const FAIL_PERSIST_CLUSTER_DISCOVERY: &str = "Failed to persist cluster discovery";
pub const FAIL_PERSIST_CLUSTER_META: &str = "Failed to persist cluster metadata";
pub const FAIL_PERSIST_EVENT: &str = "Failed to persist event";
pub const FAIL_PERSIST_NODE: &str = "Failed to persist node";
pub const FAIL_PERSIST_SHARD: &str = "Failed to persist shard";

pub const FAIL_RECENT_EVENTS: &str = "Failed to list recent events";
pub const FAIL_TOP_CLUSTERS: &str = "Failed to list biggest clusters";


pub const TOP_CLUSTERS_LIMIT: u32 = 10;

lazy_static! {
    pub static ref EXPECTED_COLLECTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert(COLLECTION_AGENTS);
        set.insert(COLLECTION_AGENTS_INFO);
        set.insert(COLLECTION_CLUSTER_META);
        set.insert(COLLECTION_DISCOVERIES);
        set.insert(COLLECTION_EVENTS);
        set.insert(COLLECTION_NODES);
        set.insert(COLLECTION_SHARDS);
        set
    };
}
