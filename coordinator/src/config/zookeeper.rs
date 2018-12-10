/// Zookeeper background cleaner configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Maximum number of nodes to delete in a single cleanup cycle.
    #[serde(default = "CleanupConfig::default_limit")]
    pub limit: usize,

    /// Maximum amount of time to wait between cleanup cycles.
    #[serde(default = "CleanupConfig::default_interval_max")]
    pub interval_max: u64,

    /// Minimum amount of time to wait between cleanup cycles.
    #[serde(default = "CleanupConfig::default_interval_min")]
    pub interval_min: u64,
}

impl Default for CleanupConfig {
    fn default() -> CleanupConfig {
        CleanupConfig {
            limit: CleanupConfig::default_limit(),
            interval_max: CleanupConfig::default_interval_max(),
            interval_min: CleanupConfig::default_interval_min(),
        }
    }
}

impl CleanupConfig {
    fn default_limit() -> usize { 1000 }
    fn default_interval_max() -> u64 { 10800 }  // 3 hours
    fn default_interval_min() -> u64 { 3600 }  // 1 hour
}


/// Zookeeper distributed coordination configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ZookeeperConfig {
    /// Zookeeper background cleaner configuration.
    #[serde(default)]
    pub cleanup: CleanupConfig,

    /// Zookeeper ensemble connection string.
    #[serde(default = "ZookeeperConfig::default_ensemble")]
    pub ensemble: String,

    /// Zookeeper session timeout (in seconds).
    #[serde(default = "ZookeeperConfig::default_timeout")]
    pub timeout: u64,
}

impl Default for ZookeeperConfig {
    fn default() -> ZookeeperConfig {
        ZookeeperConfig {
            cleanup: CleanupConfig::default(),
            ensemble: ZookeeperConfig::default_ensemble(),
            timeout: ZookeeperConfig::default_timeout(),
        }
    }
}

impl ZookeeperConfig {
    fn default_ensemble() -> String { "localhost:2181/replicante".into() }
    fn default_timeout() -> u64 { 10 }
}
