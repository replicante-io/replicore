use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::future::Future;

use rdkafka::Message;
use rdkafka::Offset::Offset;
use rdkafka::TopicPartitionList;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::Headers;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::future_producer::FutureProducer;
use rdkafka::producer::future_producer::FutureRecord;

use slog::Logger;

use super::super::TaskError;
use super::super::super::config::KafkaConfig;

use super::super::super::shared::kafka::KAFKA_TASKS_RETRY_HEADER;
use super::super::super::shared::kafka::KAFKA_TASKS_RETRY_PRODUCER;
use super::super::super::shared::kafka::consumer_config;
use super::super::super::shared::kafka::producer_config;

use super::AckStrategy;
use super::Backend;
use super::Result;
use super::Task;
use super::TaskQueue;


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
/// Kafka uses tasks to store messages in partitions.
/// As a task queue, we do not care much for that and leave it to kafka with a couple of notes:
///
///   1. Partition topics are the concurrency limit.
///      Endu users/operators need to understand kafka works this way so that scanling can
///      be done (just changing the number of threads/processes is not enough).
///   2. We map the code concept of a `TaskQueue` to three kafka topics per queue:
///      a. The queue topic (named `<queue>`).
///      b. The retry topic (named `<queue>_retry`).
///      c. The trash topic (named `<queue>_trash`).
///
/// # Retries
/// TODO
///
///
/// # Trash
/// Tasks are sent to the trash when the user (code) wants it or when a retry attemp pushes
/// the retry count to (or above) the maximum retry count.
///
/// Trashed tasks are converted to messages pushed to the trash topic and never looked at
/// again by the system.
/// End users/operators may replay the trashed tasks by copying those messages to the
/// primary kafka topic for the task (but may need to change the retry count header if they
/// want the retry functionality to work again).
pub struct Kafka {
    config: ClientConfig,
    logger: Logger,
    retry_producer: Arc<FutureProducer>,
    retry_timeout: u32,
    subscriptions: Vec<String>,
}

impl Kafka {
    pub fn new(config: KafkaConfig, logger: Logger) -> Result<Kafka> {
        let kafka_config = consumer_config(&config);
        let retry_producer = producer_config(&config, KAFKA_TASKS_RETRY_PRODUCER).create()?;
        Ok(Kafka {
            config: kafka_config,
            logger,
            retry_producer: Arc::new(retry_producer),
            retry_timeout: config.timeouts.request,
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
            retry_producer: Arc::clone(&self.retry_producer),
            retry_timeout: self.retry_timeout,
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
    retry_producer: Arc<FutureProducer>,
    retry_timeout: u32,
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

    /// Publish a new task to kafka on the given retry topic.
    ///
    /// Also used to send tasks to the trash.
    fn retry<Q: TaskQueue>(&self, topic: &str, task: Task<Q>) -> Result<()> {
        let mut headers = OwnedHeaders::new_with_capacity(task.headers.len());
        for (key, value) in &task.headers {
            headers = headers.add(key, value);
        }
        let retry_value = (task.retry_count + 1).to_string();
        headers = headers.add(KAFKA_TASKS_RETRY_HEADER, &retry_value);
        let record: FutureRecord<(), [u8]> = FutureRecord::to(topic)
            .headers(headers)
            .payload(&task.message);
        self.retry_producer.send(record, self.retry_timeout as i64)
            .wait()?.map_err(|(error, _)| error)?;
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

    fn trash(&self, task: Task<Q>) -> Result<()> {
        let topic = task.queue.name();
        let retry_topic = format!("{}_trash", topic);
        self.retry(&retry_topic, task)?;
        self.commit(&topic)?;
        self.clear_cache();
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
    retry_producer: Arc<FutureProducer>,
    retry_timeout: u32,
}

impl TaskCache {
    fn task<Q: TaskQueue>(self) -> Result<Task<Q>> {
        Ok(Task {
            ack_strategy: Arc::new(KafkaAck {
                consumer: self.consumer,
                offset: self.offset,
                partition: self.partition,
                retry_producer: self.retry_producer,
                retry_timeout: self.retry_timeout,
            }),
            headers: self.headers,
            message: self.message,
            processed: self.processed,
            queue: self.queue.parse()?,
            retry_count: self.retry_count,
        })
    }
}
