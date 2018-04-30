use replicante_agent_discovery::Config as BackendsConfig;


/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Discovery backends configuration.
    #[serde(default)]
    pub backends: BackendsConfig,

    /// Seconds to wait between discovery runs.
    #[serde(default = "Config::default_interval")]
    pub interval: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backends: BackendsConfig::default(),
            interval: Config::default_interval(),
        }
    }
}

impl Config {
    /// Default value for `interval` used by serde.
    fn default_interval() -> u64 { 60 }
}
