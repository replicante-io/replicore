use serde_derive::Deserialize;
use serde_derive::Serialize;

const DEFAULT_INTERVAL: i64 = 60;

/// Cluster settings.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterSettings {
    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Enable or disable orchestrating the cluster.
    #[serde(default = "ClusterSettings::default_enabled")]
    pub enabled: bool,

    /// Interval, in seconds, between orchestration runs.
    #[serde(default = "ClusterSettings::default_interval")]
    pub interval: i64,

    /// Namespace the cluster settings belongs to.
    pub namespace: String,
}

impl ClusterSettings {
    /// Return a synthetic ClusterSettings model from a cluster id.
    pub fn synthetic<S1, S2>(namespace: S1, cluster_id: S2) -> ClusterSettings
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let cluster_id = cluster_id.into();
        let namespace = namespace.into();
        ClusterSettings {
            cluster_id,
            enabled: ClusterSettings::default_enabled(),
            interval: ClusterSettings::default_interval(),
            namespace,
        }
    }
}

impl ClusterSettings {
    fn default_enabled() -> bool {
        true
    }

    fn default_interval() -> i64 {
        DEFAULT_INTERVAL
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
        let expected = ClusterSettings {
            cluster_id: "cluster1".to_string(),
            enabled: false,
            interval: super::DEFAULT_INTERVAL,
            namespace: "default_ns".to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_json() {
        let cluster = ClusterSettings {
            cluster_id: "cluster2".to_string(),
            enabled: true,
            interval: 4321,
            namespace: "default_ns".to_string(),
        };
        let actual = serde_json::to_string(&cluster).unwrap();
        let expected = concat!(
            r#"{"cluster_id":"cluster2","enabled":true,"interval":4321,"#,
            r#""namespace":"default_ns"}"#
        );
        assert_eq!(actual, expected);
    }
}
