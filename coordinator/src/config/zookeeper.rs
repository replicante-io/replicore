/// Zookeeper background cleaner configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Maximum number of nodes to delete in a single cleanup cycle.
    #[serde(default = "CleanupConfig::default_limit")]
    pub limit: usize,

    /// Seconds to wait between cleanup cycles.
    #[serde(default = "CleanupConfig::default_interval")]
    pub interval: u64,

    /// Number of cycles before this node will re-run an election, 0 to disable re-runs.
    ///
    /// Having the system re-run elections continuously ensures that failover procedures are
    /// exercised constantly and not just in case of errors.
    /// You do not want to discover that failover does not work when a primary fails
    /// and nothing picks it up.
    #[serde(default = "CleanupConfig::default_term")]
    pub term: u64,
}

impl Default for CleanupConfig {
    fn default() -> CleanupConfig {
        CleanupConfig {
            limit: CleanupConfig::default_limit(),
            interval: CleanupConfig::default_interval(),
            term: CleanupConfig::default_term(),
        }
    }
}

impl CleanupConfig {
    fn default_limit() -> usize { 1000 }
    fn default_interval() -> u64 { 3600 }  // 1 hour
    fn default_term() -> u64 { 6 }  // using defaults, a re-election every ~6 hours
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
