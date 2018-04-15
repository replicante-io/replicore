pub static COLLECTION_CLUSTER_LIST: &'static str = "cluster_lists";
pub static COLLECTION_CLUSTERS: &'static str = "clusters";
pub static COLLECTION_NODES: &'static str = "nodes";

pub static FAIL_CLIENT: &'static str = "Failed to configure MongoDB client";
pub static FAIL_CLUSTER_LIST_REBUILD: &'static str = "Failed to recreate cluster lists collection";
pub static FAIL_FIND_CLUSTER_META: &'static str = "Failed to find cluster metadata";
pub static FAIL_FIND_CLUSTERS: &'static str = "Failed while searching for clusters";
pub static FAIL_PERSIST_CLUSTER: &'static str = "Failed to persist cluster";
pub static FAIL_PERSIST_NODE: &'static str = "Failed to persist node";
pub static FAIL_TOP_CLUSTERS: &'static str = "Failed to list biggest clusters";

pub static TOP_CLUSTERS_LIMIT: u32 = 10;
