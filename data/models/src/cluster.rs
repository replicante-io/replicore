/// Cluster description returned by the descovery system.
///
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
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Cluster {
    pub name: String,
    pub nodes: Vec<String>,
}

impl Cluster {
    pub fn new<S>(name: S, nodes: Vec<String>) -> Cluster
        where S: Into<String>,
    {
        Cluster {
            name: name.into(),
            nodes,
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::Cluster;

    #[test]
    fn from_json() {
        let payload = r#"{"name":"test","nodes":["a","b"]}"#;
        let cluster: Cluster = serde_json::from_str(&payload).unwrap();
        let expected = Cluster::new("test", vec!["a".into(), "b".into()]);
        assert_eq!(cluster, expected);
    }

    #[test]
    fn to_json() {
        let cluster = Cluster::new("test", vec!["a".into(), "b".into()]);
        let payload = serde_json::to_string(&cluster).unwrap();
        let expected = r#"{"name":"test","nodes":["a","b"]}"#;
        assert_eq!(payload, expected);
    }
}
