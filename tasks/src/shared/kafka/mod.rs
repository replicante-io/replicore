use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;

use super::super::config::KafkaConfig;


mod constants;
mod metrics;

pub use self::constants::*;
pub use self::metrics::ClientStatsContext;
pub use self::metrics::register_metrics;


const RETRY_LEN: usize = 6;
const SKIP_LEN: usize = 8;


/// Roles a topic can have for a `Queue`.
pub enum TopicRole {
    Queue,
    Retry,
    Skip,
}


/// Set kafka configuration options common to producers and consumers.
fn common_config(config: &KafkaConfig, client_id: &str) -> ClientConfig {
    let mut kafka_config = ClientConfig::new();
    kafka_config
        .set("auto.commit.enable", "false")
        .set("auto.offset.reset", "smallest")
        .set("bootstrap.servers", &config.brokers)
        .set("client.id", client_id)
        .set("enable.auto.offset.store", "false")
        .set("enable.partition.eof", "false")
        .set("heartbeat.interval.ms", &config.heartbeat.to_string())
        .set("metadata.request.timeout.ms", &config.timeouts.metadata.to_string())
        .set("request.timeout.ms", &config.timeouts.request.to_string())
        .set("session.timeout.ms", &config.timeouts.session.to_string())
        .set("socket.timeout.ms", &config.timeouts.socket.to_string())
        .set("statistics.interval.ms", KAFKA_STATS_INTERVAL)
        .set_log_level(RDKafkaLogLevel::Debug);
    kafka_config
}


/// Set kafka configuration options for consumers (on top of common configs).
pub fn consumer_config(config: &KafkaConfig, client_id: &str, group_id: &str) -> ClientConfig {
    let mut kafka_config = common_config(config, client_id);
    kafka_config
        .set("group.id", group_id)
        .set("queued.min.messages", KAFKA_MESSAGE_QUEUE_MIN);
    kafka_config
}


/// Set kafka configuration options for producers (on top of common configs).
pub fn producer_config(config: &KafkaConfig, client_id: &str) -> ClientConfig {
    let mut kafka_config = common_config(config, client_id);
    kafka_config
        .set("queue.buffering.max.ms", "0")  // Do not buffer messages.
        .set("request.required.acks", config.ack_level.as_rdkafka_option());
    kafka_config
}


/// Parse a topic name of the given role to return a `Queue` name.
pub fn queue_from_topic(prefix: &str, topic: &str, role: TopicRole) -> String {
    let prefix_len = prefix.len() + 1;
    let topic_len = topic.len();
    match role {
        TopicRole::Queue => topic.chars().skip(prefix_len).collect(),
        TopicRole::Retry => topic.chars().skip(prefix_len)
            .take(topic_len - prefix_len - RETRY_LEN).collect(),
        TopicRole::Skip => topic.chars().skip(prefix_len)
            .take(topic_len - prefix_len - SKIP_LEN).collect(),
    }
}

/// Decorate a `Queue` name to obtain a topic name.
pub fn topic_for_queue(prefix: &str, name: &str, role: TopicRole) -> String {
    match role {
        TopicRole::Queue => format!("{}_{}", prefix, name),
        TopicRole::Retry => format!("{}_{}_retry", prefix, name),
        TopicRole::Skip => format!("{}_{}_skipped", prefix, name),
    }
}


/// Checks if the topic name is for a `TopicRole::Retry`.
pub fn topic_is_retry(topic: &str) -> bool {
    topic.ends_with("_retry")
}
