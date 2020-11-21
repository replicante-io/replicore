use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Task workers enabling configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TaskWorkers {
    /// Default status for all task workers that are not explicitly enabled/disabled.
    #[serde(default = "TaskWorkers::default_default", rename = "_default")]
    default: bool,

    /// Enable handling of cluster state refresh and aggregation tasks.
    cluster_refresh: Option<bool>,

    /// Enable handling of clusters discovery tasks.
    discover_clusters: Option<bool>,
}

impl Default for TaskWorkers {
    fn default() -> Self {
        Self {
            default: Self::default_default(),
            cluster_refresh: None,
            discover_clusters: None,
        }
    }
}

impl TaskWorkers {
    /// Default `_default` value used by serde.
    fn default_default() -> bool {
        true
    }
}

impl TaskWorkers {
    /// Check if the cluster refresh worker is enabled.
    pub fn cluster_refresh(&self) -> bool {
        self.cluster_refresh.unwrap_or(self.default)
    }

    /// Check if the discover clusters worker is enabled.
    pub fn discover_clusters(&self) -> bool {
        self.discover_clusters.unwrap_or(self.default)
    }
}
