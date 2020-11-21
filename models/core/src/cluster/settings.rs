use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Cluster settings.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterSettings {
    pub cluster_id: String,
    pub enabled: bool,
    pub namespace: String,
}

impl ClusterSettings {
    pub fn new<S1, S2>(namespace: S1, cluster_id: S2, enabled: bool) -> ClusterSettings
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let cluster_id = cluster_id.into();
        let namespace = namespace.into();
        ClusterSettings {
            cluster_id,
            enabled,
            namespace,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::ClusterSettings;

    #[test]
    fn from_json() {
        let payload =
            concat!(r#"{"cluster_id": "cluster1", "enabled": false, "namespace": "default_ns"}"#);
        let actual: ClusterSettings = serde_json::from_str(payload).unwrap();
        let expected = ClusterSettings::new("default_ns", "cluster1", false);
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_json() {
        let expected =
            concat!(r#"{"cluster_id":"cluster2","enabled":true,"namespace":"default_ns"}"#);
        let cluster = ClusterSettings::new("default_ns", "cluster2", true);
        let actual = serde_json::to_string(&cluster).unwrap();
        assert_eq!(actual, expected);
    }
}
