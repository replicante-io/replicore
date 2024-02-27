use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_externals_kafka::CommonConfig;

/// Kafka configuration options for a stream.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    #[serde(flatten)]
    pub common: CommonConfig,

    /// Prefix in front of the stream_id to derive topic names.
    #[serde(default = "KafkaConfig::default_topic_prefix")]
    pub topic_prefix: String,
}

impl KafkaConfig {
    fn default_topic_prefix() -> String {
        "stream".into()
    }
}
