use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rdkafka::Message;
use rdkafka::TopicPartitionList;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::Headers;
use rdkafka::Offset::Offset;

use slog::Logger;

use super::super::TaskError;
use super::super::super::config::KafkaConfig;

use super::AckStrategy;
use super::Backend;
use super::Result;
use super::Task;
use super::TaskQueue;


static KAFKA_TASKS_CONSUMER: &'static str = "replicante.tasks.worker";
static KAFKA_TASKS_GROUP: &'static str = "replicante.tasks.worker";
static KAFKA_TASKS_RETRY_HEADER: &'static str = "meta:task:retry";
static KAFKA_MESSAGE_QUEUE_MIN: &'static str = "5";

thread_local! {
    static THREAD_CONSUMER: RefCell<Option<Arc<BaseConsumer>>> = RefCell::new(None);
    static THREAD_TASK_CACHE: RefCell<Option<TaskCache>> = RefCell::new(None);
}


/// Fetch tasks to kafka-backed tasks queue system.
///
/// # Threads
/// This backend creates a kafka consumer for each thread that consumes from it.
/// Each thread consumes the assigned topic partitions in committed order and proceeds to the next
/// message only once a task if consumed (successfully or not) and it's offset committed.
///
///
/// # Queue <-> topics mapping
/// TODO
///
///
/// # Retries
/// TODO
///
///
/// # Trash
/// TODO
pub struct Kafka {
    config: ClientConfig,
    logger: Logger,
    subscriptions: Vec<String>,
}

impl Kafka {
    pub fn new(config: KafkaConfig, logger: Logger) -> Result<Kafka> {
        let mut kafka_config = ClientConfig::new();
        kafka_config
            .set("auto.commit.enable", "false")
            .set("auto.offset.reset", "smallest")
            .set("bootstrap.servers", &config.brokers)
            .set("client.id", KAFKA_TASKS_CONSUMER)
            .set("enable.auto.offset.store", "false")
            .set("enable.partition.eof", "false")
            .set("group.id", KAFKA_TASKS_GROUP)
            .set("heartbeat.interval.ms", &config.heartbeat.to_string())
            .set("metadata.request.timeout.ms", &config.timeouts.metadata.to_string())
            .set("queued.min.messages", KAFKA_MESSAGE_QUEUE_MIN)
            .set("session.timeout.ms", &config.timeouts.session.to_string())
            .set("socket.timeout.ms", &config.timeouts.socket.to_string())
            //TODO: Enable debug logging once we can exclude non-replicante debug by default
            //.set("debug", "consumer,cgrp,topic,fetch")
            .set_log_level(RDKafkaLogLevel::Debug);
        Ok(Kafka {
            config: kafka_config,
            logger,
            subscriptions: Vec::new(),
        })
    }
}

impl Kafka {
    /// Create a new consumer subscribed to the given partitions.
    fn consumer(&self) -> Result<BaseConsumer> {
        debug!(self.logger, "Starting new kafka workers consumer");
        let consumer: BaseConsumer = self.config.create()?;
        let names: Vec<&str> = self.subscriptions.iter().map(|n|n.as_str()).collect();
        consumer.subscribe(&names)?;
        Ok(consumer)
    }

    /// Converts an rdkafka message into a task to process.
    fn parse_message<Q: TaskQueue>(
        &self, message: BorrowedMessage, consumer: Arc<BaseConsumer>
    ) -> Result<TaskCache> {
        // Validate the message is on a supported queue.
        // The queue is stored as a string in the end because we cache it as a thread local.
        let queue: Q = message.topic().parse()?;
        let queue = queue.name();

        // Extract message metadata and payload.
        let topic = message.topic();
        let id = format!("{}:{}:{}", topic, message.partition(), message.offset());
        let payload = message.payload().ok_or_else(|| TaskError::Msg(
            format!("received task without payload (id: {})", id)
        ))?.to_vec();

        let mut headers = match message.headers() {
            None => HashMap::new(),
            Some(headers) => {
                let mut hdrs = HashMap::new();
                for idx in 0..headers.count() {
                    let (key, value) = headers.get(idx)
                        .expect("should not decode header that does not exist");
                    let key = String::from(key);
                    let value = String::from_utf8(value.to_vec())?;
                    hdrs.insert(key, value);
                }
                hdrs
            }
        };
        let retry_count = match headers.remove(KAFKA_TASKS_RETRY_HEADER) {
            None => 0,
            Some(retry_count) => retry_count.parse()?,
        };

        // Return a TaskCache instead of a task so we can store it as a thread local
        // and we ensure only one path exists to create tasks: `TaskCache::task`.
        Ok(TaskCache {
            consumer,
            headers,
            message: payload,
            offset: message.offset(),
            partition: message.partition(),
            processed: false,
            queue,
            retry_count,
        })
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn poll(&self, timeout: Duration) -> Result<Option<Task<Q>>> {
        // Check if there is a cached task to re-deliver.
        let cache = THREAD_TASK_CACHE.with(|cache| {
            cache.borrow().as_ref().map(|cache| cache.clone())
        });
        if cache.is_some() {
            // TODO: log task ID once they are introduced.
            warn!(
                self.logger,
                "Kafka thread cache contains a task, injecting delay before re-delivering"
            );
            ::std::thread::sleep(timeout);
            let task = cache.unwrap().task()?;
            return Ok(Some(task));
        }

        // Since the task cache is empty, poll the consumer.
        THREAD_CONSUMER.with(|consumer| {
            // The first time the thread polls for tasks we create a consumer.
            if consumer.borrow().is_none() {
                let new_consumer = Arc::new(self.consumer()?);
                *consumer.borrow_mut() = Some(new_consumer);
            }

            // New or old, once we have a consumer we poll it.
            let consumer = consumer.borrow();
            let poll_result = consumer.as_ref().unwrap().poll(Some(timeout));
            match poll_result {
                None => Ok(None),
                Some(Err(error)) => Err(error.into()),
                Some(Ok(message)) => {
                    let cache = self.parse_message::<Q>(
                        message, Arc::clone(consumer.as_ref().unwrap())
                    )?;
                    let task = cache.clone().task()?;
                    THREAD_TASK_CACHE.with(|cache_store| {
                        *cache_store.borrow_mut() = Some(cache);
                    });
                    Ok(Some(task))
                }
            }
        })
    }

    fn subscribe(&mut self, queue: &Q) -> Result<()> {
        self.subscriptions.push(queue.name());
        Ok(())
    }
}


/// Kafka strategy to deal with task acks and retries.
///
/// # Acks and partition re-balancing
/// Acking a task is done by committing the offset for the original kafka message.
///
/// In the presence of consumer re-balancing this could lead to lead to some re-deliveries:
///
///   1. A process starts consuming from a set of partitions.
///   2. A consumer joins or leaves so the partitions are rebalanced across consumers.
///   3. The new consumer, starting from the current committed message, for a partition that moved
///      is faster then the old consumer and commits more offsets as processed.
///   4. The old consumer for the partition completes its active task and commits an old offset.
///   5. The new consumer crashes and re-consumes messages (the old offset is reloaded on restart).
///
/// This could lead to needless retries as well as concurrent multiple executions of some messages
/// (after rebalance the new and existing consumers would get two copies of the same task)
/// but it does not lead to missed messages.
struct KafkaAck {
    consumer: Arc<BaseConsumer>,
    offset: i64,
    partition: i32,
}

impl KafkaAck {
    /// Remove the task from the thread cache so it will not be re-delivered.
    fn clear_cache(&self) {
        THREAD_TASK_CACHE.with(|cache| {
            *cache.borrow_mut() = None;
        });
    }

    /// Commit the kafka partition offset for this message as consumed.
    fn commit(&self, topic: &str) -> Result<()> {
        let mut list = TopicPartitionList::new();
        list.add_partition_offset(topic, self.partition, Offset(self.offset));
        self.consumer.commit(&list, CommitMode::Sync)?;
        Ok(())
    }
}

impl<Q: TaskQueue> AckStrategy<Q> for KafkaAck {
    fn fail(&self, _task: Task<Q>) -> Result<()> {
        // TODO: implement
        Ok(())
    }

    fn success(&self, task: Task<Q>) -> Result<()> {
        let topic = task.queue.name();
        self.commit(&topic)?;
        self.clear_cache();
        Ok(())
    }

    fn trash(&self, _task: Task<Q>) -> Result<()> {
        // TODO: implement
        Ok(())
    }
}


/// Type eroded task structure because generic thread locals are not a thing.
///
/// The problem: re-polling a kafka consumer always returns the next task,
/// even if the current offset is not committed and we failed to commit it.
/// This could lead to tasks not being processed: task is pulled, processing fails, kafka is
/// down so the task be re-emitted, kafka comes back and the next message is returned.
///
/// To avoid the problem we keep a copy of the current task for each thread around.
/// If the task offset is committed successfully we drop the cached message
/// and the next call to `Kafka::poll` will return a new task to work on.
/// If the task offset cannot be committed for any reason (other then a few expected issues
/// such as partition re-assignment) we re-deliver the same task at the next `Kafka::poll`.
#[derive(Clone)]
struct TaskCache {
    consumer: Arc<BaseConsumer>,
    headers: HashMap<String, String>,
    message: Vec<u8>,
    offset: i64,
    partition: i32,
    processed: bool,
    queue: String,
    retry_count: u8,
}

impl TaskCache {
    fn task<Q: TaskQueue>(self) -> Result<Task<Q>> {
        Ok(Task {
            ack_strategy: Arc::new(KafkaAck {
                consumer: self.consumer,
                offset: self.offset,
                partition: self.partition
            }),
            headers: self.headers,
            message: self.message,
            processed: self.processed,
            queue: self.queue.parse()?,
            retry_count: self.retry_count,
        })
    }
}
