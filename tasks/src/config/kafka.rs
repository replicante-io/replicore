/// Control the Kafka acknowledgement level for published messages.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum KafkaAckLevel {
    #[serde(rename = "all")]
    All,

    #[serde(rename = "leader_only")]
    LeaderOnly,

    #[serde(rename = "none")]
    NoAck,
}

impl KafkaAckLevel {
    /// Present the ack level as a string compatible with rdkafka client configuration.
    pub fn as_rdkafka_option(&self) -> &'static str {
        match self {
            KafkaAckLevel::All => "all",
            KafkaAckLevel::LeaderOnly => "1",
            KafkaAckLevel::NoAck => "0",
        }
    }
}

impl Default for KafkaAckLevel {
    fn default() -> KafkaAckLevel {
        KafkaAckLevel::All
    }
}


/// Kafka as a task queue configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Acknowledgement level for published messages.
    #[serde(default)]
    pub ack_level: KafkaAckLevel,

    /// Comma separated list of seed brokers.
    #[serde(default = "KafkaConfig::default_brokers")]
    pub brokers: String,

    /// Worker session keepalive heartbeat interval.
    #[serde(default = "KafkaConfig::default_heartbeat")]
    pub heartbeat: u32,

    /// Prefix to be placed in front of queue names to derive topic names.
    #[serde(default = "KafkaConfig::default_queue_preifx")]
    pub queue_prefix: String,

    /// Kafka timeout options.
    #[serde(default)]
    pub timeouts: KafkaTimeouts,
}

impl Default for KafkaConfig {
    fn default() -> KafkaConfig {
        KafkaConfig {
            ack_level: KafkaAckLevel::default(),
            brokers: KafkaConfig::default_brokers(),
            heartbeat: KafkaConfig::default_heartbeat(),
            queue_prefix: KafkaConfig::default_queue_preifx(),
            timeouts: KafkaTimeouts::default(),
        }
    }
}

impl KafkaConfig {
    fn default_brokers() -> String { "localhost:9092".into() }
    fn default_heartbeat() -> u32 { 3000 }
    fn default_queue_preifx() -> String { "task".into() }
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
