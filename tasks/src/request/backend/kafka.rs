use failure::ResultExt;
use futures::Future;

use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;

use super::super::super::config::KafkaConfig;
use super::super::super::shared::kafka::producer_config;
use super::super::super::shared::kafka::topic_for_queue;
use super::super::super::shared::kafka::ClientStatsContext;
use super::super::super::shared::kafka::TopicRole;
use super::super::super::shared::kafka::KAFKA_TASKS_ID_HEADER;
use super::super::super::shared::kafka::KAFKA_TASKS_PRODUCER;
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
    pub fn new(config: KafkaConfig) -> Result<Kafka> {
        let producer = producer_config(&config, KAFKA_TASKS_PRODUCER)
            .create_with_context(ClientStatsContext::new("request-producer"))
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        let kafka = Kafka {
            prefix: config.queue_prefix,
            producer,
            timeout: i64::from(config.timeouts.request),
        };
        Ok(kafka)
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()> {
        let mut headers = OwnedHeaders::new_with_capacity(task.headers.len());
        for (key, value) in &task.headers {
            headers = headers.add(key, value);
        }
        headers = headers.add(KAFKA_TASKS_ID_HEADER, &task.id.to_string());
        let topic = topic_for_queue(&self.prefix, &task.queue.name(), TopicRole::Queue);
        let record: FutureRecord<(), [u8]> =
            FutureRecord::to(&topic).headers(headers).payload(message);
        self.producer
            .send(record, self.timeout)
            .wait()
            .with_context(|_| ErrorKind::TaskRequest)?
            .map_err(|(error, _)| error)
            .with_context(|_| ErrorKind::TaskRequest)?;
        Ok(())
    }
}
