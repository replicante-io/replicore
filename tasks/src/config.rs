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
    // TODO
}

impl Default for KafkaConfig {
    fn default() -> KafkaConfig {
        KafkaConfig {
        }
    }
}
