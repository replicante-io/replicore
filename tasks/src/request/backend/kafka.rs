use rdkafka::config::ClientConfig;
use rdkafka::producer::base_producer::BaseRecord;
use rdkafka::producer::base_producer::DefaultProducerContext;
use rdkafka::producer::base_producer::ThreadedProducer;

//use super::super::super::TaskError;
use super::super::super::config::KafkaConfig;

use super::Backend;
use super::Result;
use super::TaskQueue;
use super::TaskRequest;


static KAFKA_TASKS_PRODUCER: &'static str = "replicante.tasks.producer";


/// Requests to kafka-backed tasks queue system.
pub struct Kafka {
    producer: ThreadedProducer<DefaultProducerContext>,
}

impl Kafka {
    pub fn new(config: KafkaConfig) -> Result<Kafka> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("client.id", KAFKA_TASKS_PRODUCER)
            .set("metadata.request.timeout.ms", &config.timeouts.metadata.to_string())
            .set("queue.buffering.max.ms", "0")  // Do not buffer messages.
            .set("request.required.acks", "all")
            .set("socket.timeout.ms", &config.timeouts.socket.to_string())
            .create()?;
        let kafka = Kafka {
            producer,
        };
        Ok(kafka)
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn request(&self, task: TaskRequest<Q>, message: &[u8]) -> Result<()> {
        let queue = task.queue.name();
        let record: BaseRecord<(), [u8], ()> = BaseRecord::to(&queue)
            .payload(message);
        self.producer.send(record).map_err(|(error, _)| error)?;
        Ok(())
    }
}
