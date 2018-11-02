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
    #[serde(flatten)]
    pub backend: Backend,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backend: Backend::default(),
        }
    }
}


/// Kafka as a task queue configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Comma separated list of seed brokers.
    #[serde(default = "KafkaConfig::default_brokers")]
    pub brokers: String,

    /// Kafka timeout options.
    #[serde(default)]
    pub timeouts: KafkaTimeouts,
}

impl Default for KafkaConfig {
    fn default() -> KafkaConfig {
        KafkaConfig {
            brokers: KafkaConfig::default_brokers(),
            timeouts: KafkaTimeouts::default(),
        }
    }
}

impl KafkaConfig {
    fn default_brokers() -> String { "localhost:9092".into() }
}


/// Kafka timeout options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaTimeouts {
    /// Timeout (in milliseconds) for non-topic requests.
    #[serde(default = "KafkaTimeouts::default_metadata")]
    pub metadata: u32,

    /// Default timeout (in milliseconds) for network requests.
    #[serde(default = "KafkaTimeouts::default_socket")]
    pub socket: u32,
}

impl Default for KafkaTimeouts {
    fn default() -> KafkaTimeouts {
        KafkaTimeouts {
            metadata: KafkaTimeouts::default_metadata(),
            socket: KafkaTimeouts::default_socket(),
        }
    }
}

impl KafkaTimeouts {
    fn default_metadata() -> u32 { 60000 }
    fn default_socket() -> u32 { 60000 }
}
