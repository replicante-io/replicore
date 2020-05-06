use failure::ResultExt;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;

use replicante_externals_kafka::headers_from_map;
use replicante_externals_kafka::ClientStatsContext;
use replicante_service_healthcheck::HealthChecks;

use super::super::super::config::KafkaConfig;
use super::super::super::shared::kafka::producer_config;
use super::super::super::shared::kafka::topic_for_queue;
use super::super::super::shared::kafka::TopicRole;
use super::super::super::shared::kafka::KAFKA_CLIENT_ID_TASKS_PRODUCER;
use super::super::super::shared::kafka::KAFKA_TASKS_ID_HEADER;
use super::super::super::ErrorKind;
use super::super::super::Result;

use super::Backend;
use super::TaskQueue;
use super::TaskRequest;

/// Requests to kafka-backed tasks queue system.
pub struct Kafka {
    prefix: String,
    producer: FutureProducer<ClientStatsContext>,
    timeout: i64,
}

impl Kafka {
    pub fn new(config: KafkaConfig, healthchecks: &mut HealthChecks) -> Result<Kafka> {
        let client_context = ClientStatsContext::new("tasks:requester");
        healthchecks.register("tasks:requester", client_context.healthcheck());
        let producer = producer_config(&config, KAFKA_CLIENT_ID_TASKS_PRODUCER)
            .create_with_context(client_context)
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        let kafka = Kafka {
            prefix: config.queue_prefix,
            producer,
            timeout: i64::from(config.common.timeouts.request),
        };
        Ok(kafka)
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()> {
        let mut headers = headers_from_map(&task.headers);
        headers = headers.add(KAFKA_TASKS_ID_HEADER, &task.id.to_string());
        let topic = topic_for_queue(&self.prefix, &task.queue.name(), TopicRole::Queue);
        let record: FutureRecord<(), [u8]> =
            FutureRecord::to(&topic).headers(headers).payload(message);
        let ack = self.producer.send(record, self.timeout);
        futures::executor::block_on(ack)
            .with_context(|_| ErrorKind::TaskRequest)?
            .map_err(|(error, _)| error)
            .with_context(|_| ErrorKind::TaskRequest)?;
        Ok(())
    }
}
