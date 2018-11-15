/// Task workers enabling configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TaskWorkers {
    /// Default status for all task workers that are not explicitly enabled/disabled.
    #[serde(default = "TaskWorkers::default_default", rename = "_default")]
    default: bool,

    /// Enable the cluster discovery to refresh a cluster state.
    discovery: Option<bool>,
}

impl Default for TaskWorkers {
    fn default() -> Self {
        Self {
            default: Self::default_default(),
            discovery: None,
        }
    }
}

impl TaskWorkers {
    /// Default `_default` value used by serde.
    fn default_default() -> bool { true }
}

impl TaskWorkers {
    /// Check if the discovery worker is enabled.
    pub fn discovery(&self) -> bool {
        self.discovery.unwrap_or(self.default)
    }
}
