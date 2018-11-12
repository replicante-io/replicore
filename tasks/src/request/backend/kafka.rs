use rdkafka::message::OwnedHeaders;
use rdkafka::producer::base_producer::BaseRecord;
use rdkafka::producer::base_producer::DefaultProducerContext;
use rdkafka::producer::base_producer::ThreadedProducer;

use super::super::super::config::KafkaConfig;
use super::super::super::shared::kafka::KAFKA_TASKS_PRODUCER;
use super::super::super::shared::kafka::producer_config;

use super::Backend;
use super::Result;
use super::TaskQueue;
use super::TaskRequest;


/// Requests to kafka-backed tasks queue system.
pub struct Kafka {
    producer: ThreadedProducer<DefaultProducerContext>,
}

impl Kafka {
    pub fn new(config: KafkaConfig) -> Result<Kafka> {
        let producer = producer_config(&config, KAFKA_TASKS_PRODUCER).create()?;
        let kafka = Kafka { producer };
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
        let record: BaseRecord<(), [u8], ()> = BaseRecord::to(&queue)
            .headers(headers)
            .payload(message);
        self.producer.send(record).map_err(|(error, _)| error)?;
        Ok(())
    }
}
