use failure::ResultExt;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::producer::FutureProducer;

use replicante_externals_kafka::ClientStatsContext;
use replicante_externals_kafka::KafkaHealthChecker;
use replicante_service_healthcheck::HealthChecks;

use crate::config::KafkaConfig;
use crate::ErrorKind;
use crate::Result;

static KAFKA_CLIENT_ID_CONSUMER: &'static str = "replicante:stream:consumer";
static KAFKA_CLIENT_ID_PRODUCER: &'static str = "replicante:stream:producer";
static KAFKA_MESSAGE_QUEUE_MIN: &'static str = "5";
static KAFKA_STATS_INTERVAL: &'static str = "1000";

/// Type alias for a BaseConsumer that has a ClientStatsContext context.
pub type StatsConsumer = BaseConsumer<ClientStatsContext>;

pub fn consumer(
    mut config: ClientConfig,
    healthcheck: KafkaHealthChecker,
    stream_id: &'static str,
    group: &str,
) -> Result<StatsConsumer> {
    let consumer_id = format!("stream:{}:{}", stream_id, group);
    config
        .set("enable.partition.eof", "false")
        .set("group.id", &consumer_id);
    let context = ClientStatsContext::with_healthcheck(consumer_id, healthcheck);
    let consumer = config
        .create_with_context(context)
        .with_context(|_| ErrorKind::BackendClientCreation)?;
    Ok(consumer)
}

pub fn consumers_config(config: &KafkaConfig) -> ClientConfig {
    let mut kafka_config = ClientConfig::new();
    kafka_config
        .set("auto.offset.reset", "smallest")
        .set("bootstrap.servers", &config.common.brokers)
        .set("client.id", KAFKA_CLIENT_ID_CONSUMER)
        // Auto commit from client store to kafka but only add offsets
        // to the store when messages are `async_ack`ed.
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set(
            "heartbeat.interval.ms",
            &config.common.heartbeat.to_string(),
        )
        .set(
            "metadata.request.timeout.ms",
            &config.common.timeouts.metadata.to_string(),
        )
        .set("queued.min.messages", KAFKA_MESSAGE_QUEUE_MIN)
        .set(
            "request.timeout.ms",
            &config.common.timeouts.request.to_string(),
        )
        .set(
            "session.timeout.ms",
            &config.common.timeouts.session.to_string(),
        )
        .set(
            "socket.timeout.ms",
            &config.common.timeouts.socket.to_string(),
        )
        .set("statistics.interval.ms", KAFKA_STATS_INTERVAL)
        .set_log_level(RDKafkaLogLevel::Debug);
    kafka_config
}

pub fn producer(
    config: &KafkaConfig,
    stream_id: &'static str,
    healthchecks: &mut HealthChecks,
) -> Result<FutureProducer<ClientStatsContext>> {
    let client_id = format!("stream:{}:producer", stream_id);
    let client_context = ClientStatsContext::new(client_id.as_str());
    healthchecks.register(client_id.as_str(), client_context.healthcheck());
    let mut kafka_config = ClientConfig::new();
    kafka_config
        .set("bootstrap.servers", &config.common.brokers)
        .set("client.id", KAFKA_CLIENT_ID_PRODUCER)
        .set(
            "metadata.request.timeout.ms",
            &config.common.timeouts.metadata.to_string(),
        )
        .set(
            "request.timeout.ms",
            &config.common.timeouts.request.to_string(),
        )
        .set(
            "socket.timeout.ms",
            &config.common.timeouts.socket.to_string(),
        )
        .set("statistics.interval.ms", KAFKA_STATS_INTERVAL)
        .set_log_level(RDKafkaLogLevel::Debug);
    let producer = kafka_config
        .create_with_context(client_context)
        .with_context(|_| ErrorKind::BackendClientCreation)?;
    Ok(producer)
}
