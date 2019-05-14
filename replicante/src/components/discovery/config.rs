use replicante_agent_discovery::Config as BackendsConfig;

/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Discovery backends configuration.
    #[serde(default)]
    pub backends: BackendsConfig,

    /// Seconds to wait between discovery runs.
    #[serde(default = "Config::default_interval")]
    pub interval: u64,

    /// Number of cycles before this node will re-run an election, 0 to disable re-runs.
    ///
    /// Having the system re-run elections continuously ensures that failover procedures are
    /// exercised constantly and not just in case of errors.
    /// You do not want to discover that failover does not work when a primary fails
    /// and nothing picks up after it.
    #[serde(default = "Config::default_term")]
    pub term: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backends: BackendsConfig::default(),
            interval: Config::default_interval(),
            term: Config::default_term(),
        }
    }
}

impl Config {
    fn default_interval() -> u64 {
        60
    }
    fn default_term() -> u64 {
        // using defaults, a re-election every ~3 hours
        10800
    }
}
