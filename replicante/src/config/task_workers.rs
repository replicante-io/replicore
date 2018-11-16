/// Task workers enabling configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TaskWorkers {
    /// Default status for all task workers that are not explicitly enabled/disabled.
    #[serde(default = "TaskWorkers::default_default", rename = "_default")]
    default: bool,

    /// Enable cluster state refresh and aggregation task processing.
    cluster_refresh: Option<bool>,
}

impl Default for TaskWorkers {
    fn default() -> Self {
        Self {
            default: Self::default_default(),
            cluster_refresh: None,
        }
    }
}

impl TaskWorkers {
    /// Default `_default` value used by serde.
    fn default_default() -> bool { true }
}

impl TaskWorkers {
    /// Check if the cluster refresh worker is enabled.
    pub fn cluster_refresh(&self) -> bool {
        self.cluster_refresh.unwrap_or(self.default)
    }
}
