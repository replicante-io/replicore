/// Cluster description returned by the descovery system.
///
/// # Cluster membership
///
/// This model descibes the expected cluster members fully.
/// The list of nodes is used to determine if nodes are down and
/// when they are added and removed from the cluster.
///
///
/// # Cluster configuration (future plan)
///
/// Any configuration option that replicante should apply to the cluster is defined in this model.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterDiscovery {
    pub cluster_id: String,
    pub nodes: Vec<String>,
}

impl ClusterDiscovery {
    pub fn new<S>(cluster_id: S, nodes: Vec<String>) -> ClusterDiscovery
    where
        S: Into<String>,
    {
        ClusterDiscovery {
            cluster_id: cluster_id.into(),
            nodes,
        }
    }
}

/// Cluster metadata generated while fetching cluster state.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterMeta {
    pub cluster_display_name: String,
    pub cluster_id: String,
    pub kinds: Vec<String>,

    // BSON does not support unsigned integers so this must be signed.
    pub agents_down: i32,
    pub nodes: i32,
    pub nodes_down: i32,
    pub shards_count: i32,
    pub shards_primaries: i32,
}

impl ClusterMeta {
    /// Create a new metadata item.
    pub fn new<S1, S2>(cluster_id: S1, cluster_display_name: S2) -> ClusterMeta
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ClusterMeta {
            agents_down: 0,
            cluster_display_name: cluster_display_name.into(),
            cluster_id: cluster_id.into(),
            kinds: Vec::new(),
            nodes: 0,
            nodes_down: 0,
            shards_count: 0,
            shards_primaries: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    mod cluster_discovery {
        use serde_json;

        use super::super::ClusterDiscovery;

        #[test]
        fn from_json() {
            let payload = r#"{"cluster_id":"test","nodes":["a","b"]}"#;
            let cluster: ClusterDiscovery = serde_json::from_str(&payload).unwrap();
            let expected = ClusterDiscovery::new("test", vec!["a".into(), "b".into()]);
            assert_eq!(cluster, expected);
        }

        #[test]
        fn to_json() {
            let cluster = ClusterDiscovery::new("test", vec!["a".into(), "b".into()]);
            let payload = serde_json::to_string(&cluster).unwrap();
            let expected = r#"{"cluster_id":"test","nodes":["a","b"]}"#;
            assert_eq!(payload, expected);
        }
    }

    mod cluster_meta {
        use serde_json;

        use super::super::ClusterMeta;

        #[test]
        fn from_json() {
            let payload = concat!(
                r#"[{"cluster_display_name":"mongo","cluster_id":"c1","kinds":["mongo"],"#,
                r#""agents_down":0,"nodes":4,"nodes_down":0,"shards_count":5,"shards_primaries":0},"#,
                r#"{"cluster_display_name":"redis","cluster_id":"c2","kinds":["redis"],"#,
                r#""agents_down":0,"nodes":2,"nodes_down":0,"shards_count":30,"shards_primaries":0}]"#
            );
            let clusters: Vec<ClusterMeta> = serde_json::from_str(payload).unwrap();
            let mut c1 = ClusterMeta::new("c1", "mongo");
            c1.kinds = vec!["mongo".into()];
            c1.nodes = 4;
            c1.shards_count = 5;
            let mut c2 = ClusterMeta::new("c2", "redis");
            c2.kinds = vec!["redis".into()];
            c2.nodes = 2;
            c2.shards_count = 30;
            let expected = vec![c1, c2];
            assert_eq!(clusters, expected);
        }

        #[test]
        fn to_json() {
            let mut c1 = ClusterMeta::new("c1", "mongo");
            c1.kinds = vec!["mongo".into()];
            c1.nodes = 4;
            let mut c2 = ClusterMeta::new("c2", "redis");
            c2.kinds = vec!["redis".into()];
            c2.nodes = 2;
            let clusters = vec![c1, c2];
            let payload = serde_json::to_string(&clusters).unwrap();
            let expected = concat!(
                r#"[{"cluster_display_name":"mongo","cluster_id":"c1","kinds":["mongo"],"#,
                r#""agents_down":0,"nodes":4,"nodes_down":0,"shards_count":0,"shards_primaries":0},"#,
                r#"{"cluster_display_name":"redis","cluster_id":"c2","kinds":["redis"],"#,
                r#""agents_down":0,"nodes":2,"nodes_down":0,"shards_count":0,"shards_primaries":0}]"#
            );
            assert_eq!(payload, expected);
        }
    }
}
