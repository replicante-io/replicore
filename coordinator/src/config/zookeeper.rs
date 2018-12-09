/// Zookeeper distributed coordination configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ZookeeperConfig {
    /// Zookeeper ensamble connection string.
    #[serde(default = "ZookeeperConfig::default_ensamble")]
    pub ensamble: String,

    /// Zookeeper session timeout (in seconds).
    #[serde(default = "ZookeeperConfig::default_timeout")]
    pub timeout: u64,
}

impl Default for ZookeeperConfig {
    fn default() -> ZookeeperConfig {
        ZookeeperConfig {
            ensamble: ZookeeperConfig::default_ensamble(),
            timeout: ZookeeperConfig::default_timeout(),
        }
    }
}

impl ZookeeperConfig {
    fn default_ensamble() -> String { "localhost:2181/replicante".into() }
    fn default_timeout() -> u64 { 10 }
}
