use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Control the Kafka acknowledgement level for published messages.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum AckLevel {
    #[serde(rename = "all")]
    All,

    #[serde(rename = "leader_only")]
    LeaderOnly,

    #[serde(rename = "none")]
    NoAck,
}

impl AckLevel {
    /// Present the ack level as a string compatible with rdkafka client configuration.
    pub fn as_rdkafka_option(&self) -> &'static str {
        match self {
            AckLevel::All => "all",
            AckLevel::LeaderOnly => "1",
            AckLevel::NoAck => "0",
        }
    }
}

impl Default for AckLevel {
    fn default() -> AckLevel {
        AckLevel::All
    }
}

/// Configuration options common to all Kafka clients.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Acknowledgement level for published messages.
    #[serde(default)]
    pub ack_level: AckLevel,

    /// Comma separated list of seed brokers.
    pub brokers: String,

    /// Client keepalive heartbeat interval.
    #[serde(default = "CommonConfig::default_heartbeat")]
    pub heartbeat: u32,

    /// Kafka timeout options.
    #[serde(default)]
    pub timeouts: Timeouts,
}

impl CommonConfig {
    fn default_heartbeat() -> u32 {
        3000
    }
}

/// Kafka timeout options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Timeouts {
    /// Timeout (in milliseconds) for non-topic requests.
    #[serde(default = "Timeouts::default_metadata")]
    pub metadata: u32,

    /// Timeout (in milliseconds) for published messages to be acknowledged.
    #[serde(default = "Timeouts::default_request")]
    pub request: u64,

    /// Timeout (in milliseconds) after which clients are presumed dead by the brokers.
    #[serde(default = "Timeouts::default_session")]
    pub session: u32,

    /// Timeout (in milliseconds) for network requests.
    #[serde(default = "Timeouts::default_socket")]
    pub socket: u32,
}

impl Default for Timeouts {
    fn default() -> Timeouts {
        Timeouts {
            metadata: Timeouts::default_metadata(),
            request: Timeouts::default_request(),
            session: Timeouts::default_session(),
            socket: Timeouts::default_socket(),
        }
    }
}

impl Timeouts {
    fn default_metadata() -> u32 {
        60000
    }
    fn default_request() -> u64 {
        5000
    }
    fn default_session() -> u32 {
        10000
    }
    fn default_socket() -> u32 {
        60000
    }
}
