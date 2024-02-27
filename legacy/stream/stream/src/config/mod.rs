use serde_derive::Deserialize;
use serde_derive::Serialize;

mod kafka;

pub use self::kafka::KafkaConfig;

/// Stream configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend", content = "options")]
pub enum StreamConfig {
    /// Use kafka as the stream platform (recommended).
    #[serde(rename = "kafka")]
    Kafka(KafkaConfig),
}
