use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_externals_kafka::CommonConfig;

/// Kafka as a task queue configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    #[serde(flatten)]
    pub common: CommonConfig,

    /// Number of attempts to commit offsets before giving up and recreating the client.
    #[serde(default = "KafkaConfig::default_commit_retries")]
    pub commit_retries: u8,

    /// Prefix to be placed in front of queue names to derive topic names.
    #[serde(default = "KafkaConfig::default_queue_preifx")]
    pub queue_prefix: String,
}

impl KafkaConfig {
    fn default_commit_retries() -> u8 {
        5
    }
    fn default_queue_preifx() -> String {
        "task".into()
    }
}
