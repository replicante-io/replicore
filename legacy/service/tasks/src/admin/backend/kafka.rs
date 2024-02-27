use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use failure::ResultExt;
use slog::Logger;

use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::Headers;
use rdkafka::Message;

use replicante_externals_kafka::ClientStatsContext;
use replicante_util_rndid::RndId;

use super::super::AdminBackend;
use super::super::TasksIter;
use crate::config::KafkaConfig;
use crate::shared::kafka::consumer_config;
use crate::shared::kafka::queue_from_topic;
use crate::shared::kafka::topic_for_queue;
use crate::shared::kafka::TopicRole;
use crate::shared::kafka::KAFKA_ADMIN_CONSUMER;
use crate::shared::kafka::KAFKA_ADMIN_GROUP;
use crate::shared::kafka::KAFKA_TASKS_ID_HEADER;
use crate::shared::kafka::KAFKA_TASKS_RETRY_HEADER;
use crate::worker::AckStrategy;
use crate::Error;
use crate::ErrorKind;
use crate::Result;
use crate::Task;
use crate::TaskId;
use crate::TaskQueue;

const TIMEOUT_MS_POLL: u64 = 500;

/// Type alias for a BaseConsumer that has a ClientStatsContext context.
type BaseStatsConsumer = BaseConsumer<ClientStatsContext>;

/// Admin tasks backend for kafka tasks.
pub struct Kafka {
    config: KafkaConfig,
}

impl Kafka {
    pub fn new(_logger: Logger, config: KafkaConfig) -> Result<Kafka> {
        Ok(Kafka { config })
    }
}

impl<Q: TaskQueue> AdminBackend<Q> for Kafka {
    fn scan(&self, queue: Q) -> Result<TasksIter<Q>> {
        let queue_name = queue.name();

        // Generate consumer IDs.
        let client_id = format!("{}:{}", KAFKA_ADMIN_CONSUMER, queue_name);
        let group_id = format!("{}:{}:{}", KAFKA_ADMIN_GROUP, queue_name, RndId::new());
        let stats_id = format!("tasks:admin:{}:consumer", queue_name);
        let kafka_config = consumer_config(&self.config, &client_id, &group_id);

        // Create consumer and subscribe to queue's topics.
        let consumer: BaseStatsConsumer = kafka_config
            .create_with_context(ClientStatsContext::new(stats_id))
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        consumer
            .subscribe(&[
                &topic_for_queue(&self.config.queue_prefix, &queue_name, TopicRole::Queue),
                &topic_for_queue(&self.config.queue_prefix, &queue_name, TopicRole::Retry),
            ])
            .with_context(|_| ErrorKind::TaskSubscription)?;
        Ok(TasksIter(Box::new(KafkaIter {
            _queue: ::std::marker::PhantomData,
            consumer,
            prefix: self.config.queue_prefix.clone(),
            stream_started: false,
        })))
    }

    fn version(&self) -> Result<String> {
        Ok("Kafka (version not reported)".into())
    }
}

/// Iterator over kafka stored tasks.
struct KafkaIter<Q: TaskQueue> {
    _queue: ::std::marker::PhantomData<Q>,
    consumer: BaseStatsConsumer,
    prefix: String,
    stream_started: bool,
}

impl<Q: TaskQueue> KafkaIter<Q> {
    fn parse_message(&self, message: BorrowedMessage) -> Result<Task<Q>> {
        // Validate the message is on a supported queue.
        // The queue is stored as a string in the end because we cache it as a thread local.
        let queue = queue_from_topic(&self.prefix, message.topic(), TopicRole::Queue);
        let queue = queue
            .parse::<Q>()
            .with_context(|_| ErrorKind::QueueNameInvalid(queue))?;

        // Extract message metadata and payload.
        let topic = message.topic();
        let message_id = format!("{}:{}:{}", topic, message.partition(), message.offset());
        let payload = message
            .payload()
            .ok_or(ErrorKind::TaskNoPayload(message_id))?
            .to_vec();
        let mut headers = match message.headers() {
            None => HashMap::new(),
            Some(headers) => {
                let mut hdrs = HashMap::new();
                for idx in 0..headers.count() {
                    let (key, value) = headers
                        .get(idx)
                        .expect("should not decode header that does not exist");
                    let key = String::from(key);
                    let value = String::from_utf8(value.to_vec()).with_context(|_| {
                        ErrorKind::TaskHeaderInvalid(key.clone(), "<not-utf8>".into())
                    })?;
                    hdrs.insert(key, value);
                }
                hdrs
            }
        };
        let id: TaskId = match headers.remove(KAFKA_TASKS_ID_HEADER) {
            None => return Err(ErrorKind::TaskNoId.into()),
            Some(id) => id
                .parse::<TaskId>()
                .with_context(|_| ErrorKind::TaskInvalidID(id))?,
        };
        let retry_count = match headers.remove(KAFKA_TASKS_RETRY_HEADER) {
            None => 0,
            Some(retry_count) => retry_count.parse::<u8>().with_context(|_| {
                let key = KAFKA_TASKS_RETRY_HEADER.to_string();
                ErrorKind::TaskHeaderInvalid(key, retry_count)
            })?,
        };

        // Return a special task that can't be acked and does not panic if not processed.
        Ok(Task {
            ack_strategy: Arc::new(ForbidAck {}),
            headers,
            id,
            message: payload,
            processed: true,
            queue,
            retry_count,
        })
    }
}

impl<Q: TaskQueue> Iterator for KafkaIter<Q> {
    type Item = Result<Task<Q>>;
    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(60) {
            match self.consumer.poll(Duration::from_millis(TIMEOUT_MS_POLL)) {
                // Keep trying to fetch messages if we never got any.
                // This allows kafka to perform group balancing and other initialisation tasks.
                None if !self.stream_started => continue,
                None => return None,
                Some(Err(error)) => {
                    self.stream_started = true;
                    let error = Err(error)
                        .with_context(|_| ErrorKind::FetchError)
                        .map_err(Error::from);
                    return Some(error);
                }
                Some(Ok(message)) => {
                    self.stream_started = true;
                    return Some(self.parse_message(message));
                }
            }
        }
        // If kafka did not return anything in more then WAIT_TASKS_START
        // give up and assume the topics are empty.
        None
    }
}

/// Acks are not allowed while scanning kafka tasks.
///
/// It would be impossible to ack a task and not all the other ones.
struct ForbidAck {}

impl<Q: TaskQueue> AckStrategy<Q> for ForbidAck {
    fn fail(&self, _: Task<Q>) -> Result<()> {
        Err(ErrorKind::ScanCannotAck("fail").into())
    }

    fn skip(&self, _: Task<Q>) -> Result<()> {
        Err(ErrorKind::ScanCannotAck("skip").into())
    }

    fn success(&self, _: Task<Q>) -> Result<()> {
        Err(ErrorKind::ScanCannotAck("succeed").into())
    }
}
