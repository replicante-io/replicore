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
    fn default_threads_count() -> u16 { 8 * (::num_cpus::get() as u16) }
}


/// Kafka as a task queue configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Comma separated list of seed brokers.
    #[serde(default = "KafkaConfig::default_brokers")]
    pub brokers: String,

    /// Worker session keepalive heartbeat interval.
    #[serde(default = "KafkaConfig::default_heartbeat")]
    pub heartbeat: u32,

    /// Kafka timeout options.
    #[serde(default)]
    pub timeouts: KafkaTimeouts,
}

impl Default for KafkaConfig {
    fn default() -> KafkaConfig {
        KafkaConfig {
            brokers: KafkaConfig::default_brokers(),
            heartbeat: KafkaConfig::default_heartbeat(),
            timeouts: KafkaTimeouts::default(),
        }
    }
}

impl KafkaConfig {
    fn default_brokers() -> String { "localhost:9092".into() }
    fn default_heartbeat() -> u32 { 3000 }
}


/// Kafka timeout options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaTimeouts {
    /// Timeout (in milliseconds) for non-topic requests.
    #[serde(default = "KafkaTimeouts::default_metadata")]
    pub metadata: u32,

    /// Timeout (in milliseconds) for tasks to be acknowledged.
    #[serde(default = "KafkaTimeouts::default_request")]
    pub request: u32,

    /// Timeout (in milliseconds) after which workers are presumed dead by the brokers.
    #[serde(default = "KafkaTimeouts::default_session")]
    pub session: u32,

    /// Timeout (in milliseconds) for network requests.
    #[serde(default = "KafkaTimeouts::default_socket")]
    pub socket: u32,
}

impl Default for KafkaTimeouts {
    fn default() -> KafkaTimeouts {
        KafkaTimeouts {
            metadata: KafkaTimeouts::default_metadata(),
            request: KafkaTimeouts::default_request(),
            session: KafkaTimeouts::default_session(),
            socket: KafkaTimeouts::default_socket(),
        }
    }
}

impl KafkaTimeouts {
    fn default_metadata() -> u32 { 60000 }
    fn default_request() -> u32 { 5000 }
    fn default_session() -> u32 { 10000 }
    fn default_socket() -> u32 { 60000 }
}
