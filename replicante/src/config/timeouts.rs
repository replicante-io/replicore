/// Replicante timeouts configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TimeoutsConfig {
    /// Time after which API requests to agents are failed.
    #[serde(default = "TimeoutsConfig::default_agents_api")]
    pub agents_api: u64,
}

impl Default for TimeoutsConfig {
    fn default() -> TimeoutsConfig {
        TimeoutsConfig {
            agents_api: Self::default_agents_api(),
        }
    }
}

impl TimeoutsConfig {
    /// Default value for `agents_api` used by serde.
    fn default_agents_api() -> u64 {
        15
    }
}
