use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;

use super::super::config::KafkaConfig;


mod constants;
mod metrics;

pub use self::constants::*;
pub use self::metrics::ClientStatsContext;
pub use self::metrics::register_metrics;


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
pub fn consumer_config(config: &KafkaConfig) -> ClientConfig {
    let mut kafka_config = common_config(config, KAFKA_TASKS_CONSUMER);
    kafka_config
        .set("group.id", KAFKA_TASKS_GROUP)
        //TODO: Enable debug logging once we can exclude non-replicante debug by default
        //.set("debug", "consumer,cgrp,topic,fetch")
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
