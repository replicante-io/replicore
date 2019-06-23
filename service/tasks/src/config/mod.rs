use serde_derive::Deserialize;
use serde_derive::Serialize;

mod kafka;

pub use self::kafka::KafkaConfig;

/// Task queue backend configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options")]
pub enum Backend {
    /// Use kafka as a task system (recommended, default).
    #[serde(rename = "kafka")]
    Kafka(KafkaConfig),
}

/// Tasks configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub backend: Backend,

    /// Number of task processing threads to spawn
    #[serde(default = "Config::default_threads_count")]
    pub threads_count: u16,
}

impl Config {
    fn default_threads_count() -> u16 {
        ::num_cpus::get() as u16
    }
}
