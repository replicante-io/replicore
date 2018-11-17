use futures::Future;

use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;

use super::super::super::config::KafkaConfig;
use super::super::super::shared::kafka::ClientStatsContext;
use super::super::super::shared::kafka::KAFKA_TASKS_PRODUCER;
use super::super::super::shared::kafka::producer_config;

use super::Backend;
use super::Result;
use super::TaskQueue;
use super::TaskRequest;


/// Requests to kafka-backed tasks queue system.
pub struct Kafka {
    producer: FutureProducer<ClientStatsContext>,
    timeout: i64,
}

impl Kafka {
    pub fn new(config: KafkaConfig) -> Result<Kafka> {
        let producer = producer_config(&config, KAFKA_TASKS_PRODUCER)
            .create_with_context(ClientStatsContext::new("request-producer"))?;
        let kafka = Kafka {
            producer,
            timeout: config.timeouts.request as i64,
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
        let queue = task.queue.name();
        let record: FutureRecord<(), [u8]> = FutureRecord::to(&queue)
            .headers(headers)
            .payload(message);
        self.producer.send(record, self.timeout)
            .wait()?.map_err(|(error, _)| error)?;
        Ok(())
    }
}
