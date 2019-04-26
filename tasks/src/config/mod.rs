mod kafka;

pub use self::kafka::KafkaConfig;


/// Task queue backend configuration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options", deny_unknown_fields)]
pub enum Backend {
    /// Use kafka as a task system (recommended, default).
    #[serde(rename = "kafka")]
    Kafka(KafkaConfig),
}

impl Default for Backend {
    fn default() -> Backend {
        Backend::Kafka(KafkaConfig::default())
    }
}


/// Tasks configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub backend: Backend,

    /// Number of task processing threads to spawn
    #[serde(default = "Config::default_threads_count")]
    pub threads_count: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backend: Backend::default(),
            threads_count: Config::default_threads_count(),
        }
    }
}

impl Config {
    fn default_threads_count() -> u16 {
        ::num_cpus::get() as u16
    }
}
