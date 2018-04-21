//pub static COLLECTION_AGENTS: &'static str = "agents";
//pub static COLLECTION_AGENTS_INFO: &'static str = "agents_info";
pub static COLLECTION_CLUSTER_META: &'static str = "clusters_meta";
pub static COLLECTION_DISCOVERIES: &'static str = "discoveries";
//pub static COLLECTION_EVENTS: &'static str = "events";
pub static COLLECTION_NODES: &'static str = "nodes";
//pub static COLLECTION_SHARDS: &'static str = "shards";


pub static FAIL_CLIENT: &'static str = "Failed to configure MongoDB client";

pub static FAIL_FIND_CLUSTER_DISCOVERY: &'static str = "Failed to find cluster discovery record";
pub static FAIL_FIND_CLUSTER_META: &'static str = "Failed to find cluster metadata";
pub static FAIL_FIND_CLUSTERS: &'static str = "Failed while searching for clusters";

pub static FAIL_PERSIST_CLUSTER_DISCOVERY: &'static str = "Failed to persist cluster discovery";
pub static FAIL_PERSIST_CLUSTER_META: &'static str = "Failed to persist cluster metadata";
pub static FAIL_PERSIST_NODE: &'static str = "Failed to persist node";

pub static FAIL_TOP_CLUSTERS: &'static str = "Failed to list biggest clusters";


pub static TOP_CLUSTERS_LIMIT: u32 = 10;
