use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use failure::ResultExt;
use futures::future::Future;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::OwnedHeaders;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::future_producer::FutureProducer;
use rdkafka::producer::future_producer::FutureRecord;
use rdkafka::Message;
use rdkafka::Offset::Offset;
use rdkafka::TopicPartitionList;
use slog::debug;
use slog::warn;
use slog::Logger;

use replicante_externals_kafka::headers_to_map;
use replicante_externals_kafka::ClientStatsContext;
use replicante_externals_kafka::KafkaHealthChecker;
use replicante_service_healthcheck::HealthChecks;

use super::super::super::config::KafkaConfig;
use super::super::super::shared::kafka::consumer_config;
use super::super::super::shared::kafka::producer_config;
use super::super::super::shared::kafka::queue_from_topic;
use super::super::super::shared::kafka::topic_for_queue;
use super::super::super::shared::kafka::topic_is_retry;
use super::super::super::shared::kafka::TopicRole;
use super::super::super::shared::kafka::KAFKA_CLIENT_ID_CONSUMER;
use super::super::super::shared::kafka::KAFKA_CLIENT_ID_RETRY_PRODUCER;
use super::super::super::shared::kafka::KAFKA_TASKS_GROUP;
use super::super::super::shared::kafka::KAFKA_TASKS_ID_HEADER;
use super::super::super::shared::kafka::KAFKA_TASKS_RETRY_HEADER;
use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::super::TaskId;

use super::AckStrategy;
use super::Backend;
use super::Task;
use super::TaskQueue;

/// Type alias for a BaseConsumer that has a ClientStatsContext context.
type BaseStatsConsumer = BaseConsumer<ClientStatsContext>;

thread_local! {
    // Task rerty caches.
    static THREAD_RETRY_CACHE: RefCell<Option<RetryCache>> = RefCell::new(None);
    static THREAD_RETRY_CLEAR: RefCell<bool> = RefCell::new(false);
    static THREAD_RETRY_CONSUMER: RefCell<Option<Arc<BaseStatsConsumer>>> = RefCell::new(None);

    // Task consumer caches.
    static THREAD_TASK_CACHE: RefCell<Option<TaskCache>> = RefCell::new(None);
    static THREAD_TASK_CLEAR: RefCell<bool> = RefCell::new(false);
    static THREAD_TASK_CONSUMER: RefCell<Option<Arc<BaseStatsConsumer>>> = RefCell::new(None);
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
///      c. The skipped topic (named `<queue>_skipped`).
///
///
/// # Offset commits
/// Message offsets are committed in order and only after a task is fully processed
/// (either acked or moved to a retry/skipped topic).
///
/// The consumer start consuming messages AT the committed offset, not AFTER it.
/// This means that if we conume and commit offset 16 and the process is restarted
/// it will be restarted at offset 16 instead of 17.
///
/// As a result we commit offset + 1 after each message is processed
/// so that consumers can resume processing tasks without duplicates.
///
///
/// # Retries
/// Failed tasks are copied onto a retry topic.
/// The retry topics are checked before the main topics are to see if any task needs to be
/// retired and if it is time to do so.
///
/// When a task needs to be retried it is copied back to the main topic and its offset committed.
/// Tasks are NOT processed directly off of the retry topic.
/// The delay for task retry is fixed (and can't be a backoff delay) because that would
/// require a topic for each backoff level, which is too complex for now.
///
///
/// # Skipped tasks
/// Tasks are skipped when the user (code) wants it or when a retry attemp pushes
/// the retry count to (or above) the maximum retry count.
///
/// Skipped tasks are converted to messages pushed to the skipped topic
/// and never looked at again by the system.
/// End users/operators may replay the skipped tasks by copying those messages to the
/// primary kafka topic for the task (but may need to change the retry count header if they
/// want the retry functionality to work again).
pub struct Kafka {
    commit_retries: u8,
    config: ClientConfig,
    health: KafkaHealthChecker,
    logger: Logger,
    prefix: String,
    retry_producer: Arc<FutureProducer<ClientStatsContext>>,
    retry_subscriptions: Vec<String>,
    retry_timeout: u32,
    subscriptions: Vec<String>,
}

impl Kafka {
    pub fn new(
        config: KafkaConfig,
        logger: Logger,
        healthchecks: &mut HealthChecks,
    ) -> Result<Kafka> {
        let health = KafkaHealthChecker::new();
        healthchecks.register("tasks:workers", health.clone());
        let client_context =
            ClientStatsContext::with_healthcheck("tasks:workers:retrier", health.clone());
        let retry_producer = producer_config(&config, KAFKA_CLIENT_ID_RETRY_PRODUCER)
            .create_with_context(client_context)
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        let kafka_config = consumer_config(&config, KAFKA_CLIENT_ID_CONSUMER, KAFKA_TASKS_GROUP);
        Ok(Kafka {
            commit_retries: config.commit_retries,
            config: kafka_config,
            health,
            logger,
            prefix: config.queue_prefix,
            retry_producer: Arc::new(retry_producer),
            retry_subscriptions: Vec::new(),
            retry_timeout: config.common.timeouts.request,
            subscriptions: Vec::new(),
        })
    }
}

impl Kafka {
    /// Poll the *_retry submissions and re-enqueue tasks if the time is right.
    fn check_retries<Q: TaskQueue>(&self, timeout: Duration) -> Result<()> {
        THREAD_RETRY_CONSUMER.with(|consumer| {
            // The first time the thread polls for tasks we create a consumer.
            if consumer.borrow().is_none() {
                let new_consumer = Arc::new(self.consumer(&self.retry_subscriptions)?);
                *consumer.borrow_mut() = Some(new_consumer);
            }
            // New or old, once we have a consumer to use.
            let consumer = consumer.borrow();
            let consumer = consumer.as_ref().unwrap();

            // Check if there is a cached task to (possibly) re-process.
            let early_return = THREAD_RETRY_CACHE.with(|cache| -> Result<bool> {
                let mut clear_cache = false;
                if let Some(retry_cache) = cache.borrow_mut().as_mut() {
                    let retried = self.retry_message::<Q>(consumer, retry_cache)?;
                    if !retried {
                        // The task still needs to be processed so retry later.
                        return Ok(true);
                    }
                    clear_cache = true;
                }
                if clear_cache {
                    *cache.borrow_mut() = None;
                    debug!(self.logger, "Scheduled task from retry cache");
                }
                Ok(false)
            })?;
            if early_return {
                // We have a task in the cache that needs to be retried later so move on for now.
                return Ok(());
            }

            // Since the task retry cache is empty, poll the consumer.
            let mut timeout = timeout;
            loop {
                let start = Instant::now();
                if self.poll_retries::<Q>(consumer, timeout)? {
                    // Return early if there was no task to retry.
                    return Ok(());
                }
                let mut duration = start.elapsed();
                // Ensure the duration is never zero so we can avoid endless loops.
                if duration == Duration::from_micros(0) {
                    duration = Duration::from_micros(1);
                }
                timeout = match timeout.checked_sub(duration) {
                    None => return Ok(()),
                    // Still exit if the timeout is 0.
                    Some(t) if t == Duration::from_micros(0) => return Ok(()),
                    Some(t) => t,
                };
            }
        })
    }

    /// Create a new consumer subscribed to the given partitions.
    fn consumer(&self, subscriptions: &[String]) -> Result<BaseStatsConsumer> {
        debug!(self.logger, "Starting new kafka consumer"; "subscriptions" => ?subscriptions);
        let consumer_role = format!("tasks:consumer:worker:{:?}", ::std::thread::current().id());
        let context = ClientStatsContext::with_healthcheck(consumer_role, self.health.clone());
        let consumer: BaseStatsConsumer = self
            .config
            .create_with_context(context)
            .with_context(|_| ErrorKind::BackendClientCreation)?;
        let names: Vec<&str> = subscriptions.iter().map(String::as_str).collect();
        consumer
            .subscribe(&names)
            .with_context(|_| ErrorKind::TaskSubscription)?;
        Ok(consumer)
    }

    /// Converts an rdkafka message into a task to process.
    fn parse_message<Q: TaskQueue>(
        &self,
        message: BorrowedMessage,
        consumer: Arc<BaseStatsConsumer>,
    ) -> Result<TaskCache> {
        // Validate the message is on a supported queue.
        // The queue is stored as a string in the end because we cache it as a thread local.
        let queue = queue_from_topic(&self.prefix, message.topic(), TopicRole::Queue);
        let queue = queue
            .parse::<Q>()
            .with_context(|_| ErrorKind::QueueNameInvalid(queue))?;
        let queue = queue.name();

        // Extract message metadata and payload.
        let topic = message.topic();
        let message_id = format!("{}:{}:{}", topic, message.partition(), message.offset());
        let payload = message
            .payload()
            .ok_or_else(|| ErrorKind::TaskNoPayload(message_id))?
            .to_vec();
        let mut headers = headers_to_map(message.headers()).with_context(|e| {
            ErrorKind::TaskHeaderInvalid(e.header.clone(), "<not-utf8>".into())
        })?;
        let id: TaskId = match headers.remove(KAFKA_TASKS_ID_HEADER) {
            None => return Err(ErrorKind::TaskNoId.into()),
            Some(id) => id
                .parse::<TaskId>()
                .with_context(|_| ErrorKind::TaskInvalidID(id))?,
        };
        let retry_count = match headers.remove(KAFKA_TASKS_RETRY_HEADER) {
            None => 0,
            Some(retry_count) => retry_count.parse::<u8>().with_context(|_| {
                let header = KAFKA_TASKS_RETRY_HEADER.to_string();
                ErrorKind::TaskHeaderInvalid(header, retry_count)
            })?,
        };

        // Return a TaskCache instead of a task so we can store it as a thread local
        // and we ensure only one path exists to create tasks: `TaskCache::task`.
        Ok(TaskCache {
            commit_attempts: Rc::new(RefCell::new(0)),
            commit_max_attemts: self.commit_retries,
            consumer,
            headers,
            id,
            logger: self.logger.clone(),
            message: payload,
            offset: message.offset(),
            partition: message.partition(),
            prefix: self.prefix.clone(),
            queue,
            retry_count,
            retry_producer: Arc::clone(&self.retry_producer),
            retry_published: Rc::new(RefCell::new(false)),
            retry_timeout: self.retry_timeout,
            skipped_published: Rc::new(RefCell::new(false)),
        })
    }

    /// Check if there is a retry task to consume (and retry or cache).
    ///
    /// This method returns `true` if the last inspected task can't be re-tried yet
    /// (or there were no tasks to process).
    /// Think of this as the answer to the question "are the retry checks done for now?".
    fn poll_retries<Q: TaskQueue>(
        &self,
        consumer: &BaseStatsConsumer,
        timeout: Duration,
    ) -> Result<bool> {
        let poll_result = consumer.poll(Some(timeout));
        match poll_result {
            None => Ok(true),
            Some(Err(error)) => Err(error)
                .with_context(|_| ErrorKind::FetchError)
                .map_err(Error::from),
            Some(Ok(message)) => {
                let message = message.detach();
                // Always cache the message in case of errors while processing it.
                let retried = THREAD_RETRY_CACHE.with(|cache| {
                    *cache.borrow_mut() = Some(RetryCache::new(message));
                    self.retry_message::<Q>(consumer, cache.borrow_mut().as_mut().unwrap())
                })?;
                if retried {
                    // Clear the cache since the task was retired correctly.
                    THREAD_RETRY_CACHE.with(|cache| *cache.borrow_mut() = None);
                    Ok(false)
                } else {
                    // Not yet time to retry this task, leve it cached and stop checks for now.
                    debug!(
                        self.logger,
                        "Found retry task that could not yet be scheduled"
                    );
                    Ok(true)
                }
            }
        }
    }

    /// Schedule a task from the retry topic to the main topic and commit its retry offset.
    ///
    /// Returns `true` if the task was rescheduled and `false` otherwise
    fn retry_message<Q: TaskQueue>(
        &self,
        consumer: &BaseStatsConsumer,
        retry_cache: &mut RetryCache,
    ) -> Result<bool> {
        // Determine topic to retry to and the queue the task belong to.
        let message = &retry_cache.message;
        let topic = message.topic();
        if !topic_is_retry(topic) {
            panic!("Attempting to retry task from non _retry topic '{}'", topic);
        }
        let queue = queue_from_topic(&self.prefix, topic, TopicRole::Retry);
        let queue = queue
            .parse::<Q>()
            .with_context(|_| ErrorKind::QueueNameInvalid(queue))?;
        let topic = topic_for_queue(&self.prefix, &queue.name(), TopicRole::Queue);

        // Check if the task has reached the retry delay.
        let timestamp = message.timestamp().to_millis().unwrap_or(0);
        let now = ::rdkafka::message::Timestamp::now().to_millis().unwrap();
        let retry_delay = queue.retry_delay();
        let retry_delay = retry_delay.as_secs() * 1000 + u64::from(retry_delay.subsec_millis());
        let retry = (now - timestamp).abs() as u64 >= retry_delay;

        // If so, re-publish the message to the task queue.
        if retry {
            // Avoid knowingly injecting dupilcates by not publishing if we already did
            // but committing the offset later failed.
            if !retry_cache.published {
                let mut record: FutureRecord<(), [u8]> = FutureRecord::to(&topic);
                if let Some(headers) = message.headers() {
                    record = record.headers(headers.clone());
                }
                if let Some(payload) = message.payload() {
                    record = record.payload(payload);
                }
                self.retry_producer
                    .send(record, i64::from(self.retry_timeout))
                    .wait()
                    .with_context(|_| ErrorKind::RetryEnqueue)?
                    .map_err(|(error, _)| error)
                    .with_context(|_| ErrorKind::RetryEnqueue)?;
                retry_cache.published = true;
            }
            let mut list = TopicPartitionList::new();
            list.add_partition_offset(
                message.topic(),
                message.partition(),
                Offset(message.offset() + 1),
            );
            if retry_cache.attempts >= self.commit_retries {
                let message_id = format!(
                    "{}:{}:{}",
                    message.topic(),
                    message.partition(),
                    message.offset()
                );
                warn!(self.logger, "Stuck trying to commit replyed task"; "id" => &message_id);
                THREAD_RETRY_CLEAR.with(|retry| *retry.borrow_mut() = true);
                return Err(ErrorKind::CommitRetryStuck(message_id).into());
            }
            retry_cache.attempts += 1;
            consumer
                .commit(&list, CommitMode::Sync)
                .with_context(|_| ErrorKind::CommitFailed)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn poll(&self, timeout: Duration) -> Result<Option<Task<Q>>> {
        // Drop all caches and clients if we flaged for clear.
        THREAD_RETRY_CLEAR.with(|clear| {
            if *clear.borrow() {
                THREAD_RETRY_CACHE.with(|cache| *cache.borrow_mut() = None);
                THREAD_RETRY_CONSUMER.with(|cache| *cache.borrow_mut() = None);
                *clear.borrow_mut() = false;
                debug!(self.logger, "Cleared kafka worker retry cache");
            }
        });
        THREAD_TASK_CLEAR.with(|clear| {
            if *clear.borrow() {
                THREAD_TASK_CACHE.with(|cache| *cache.borrow_mut() = None);
                THREAD_TASK_CONSUMER.with(|cache| *cache.borrow_mut() = None);
                *clear.borrow_mut() = false;
                debug!(self.logger, "Cleared kafka worker task cache");
            }
        });

        // Check if any task needs to be rertied.
        self.check_retries::<Q>(timeout)?;

        // Check if there is a cached task to re-deliver.
        let cache = THREAD_TASK_CACHE.with(|cache| cache.borrow().as_ref().cloned());
        if let Some(cache) = cache {
            let task = cache.task()?;
            warn!(
                self.logger,
                "Kafka thread cache contains a task, injecting delay before re-delivering";
                "task-id" => %task.id()
            );
            ::std::thread::sleep(timeout);
            return Ok(Some(task));
        }

        // Since the task cache is empty, poll the consumer.
        THREAD_TASK_CONSUMER.with(|consumer| {
            // The first time the thread polls for tasks we create a consumer.
            if consumer.borrow().is_none() {
                let new_consumer = Arc::new(self.consumer(&self.subscriptions)?);
                *consumer.borrow_mut() = Some(new_consumer);
            }

            // New or old, once we have a consumer we poll it.
            let consumer = consumer.borrow();
            let poll_result = consumer.as_ref().unwrap().poll(Some(timeout));
            match poll_result {
                None => Ok(None),
                Some(Err(error)) => Err(error)
                    .with_context(|_| ErrorKind::FetchError)
                    .map_err(Error::from),
                Some(Ok(message)) => {
                    let cache =
                        self.parse_message::<Q>(message, Arc::clone(consumer.as_ref().unwrap()))?;
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
        let queue_name = queue.name();
        let retry = topic_for_queue(&self.prefix, &queue_name, TopicRole::Retry);
        self.retry_subscriptions.push(retry);
        self.subscriptions
            .push(topic_for_queue(&self.prefix, &queue_name, TopicRole::Queue));
        Ok(())
    }

    fn worker_cleanup(&self) {
        // The kafka consumer logs in its Drop::drop implementation.
        // That in turn uses scope_log and Thread Local Store (TLS).
        // Because the consumer is being dopped as the thread is exiting, TLS access panics.
        // We explicitly drop the consumers here to avoid this issue.
        THREAD_RETRY_CACHE.with(|cache| cache.borrow_mut().take());
        THREAD_RETRY_CONSUMER.with(|consumer| consumer.borrow_mut().take());
        THREAD_TASK_CACHE.with(|cache| cache.borrow_mut().take());
        THREAD_TASK_CONSUMER.with(|consumer| consumer.borrow_mut().take());
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
    commit_attempts: Rc<RefCell<u8>>,
    commit_max_attemts: u8,
    consumer: Arc<BaseStatsConsumer>,
    logger: Logger,
    offset: i64,
    partition: i32,
    prefix: String,
    retry_producer: Arc<FutureProducer<ClientStatsContext>>,
    retry_published: Rc<RefCell<bool>>,
    retry_timeout: u32,
    skipped_published: Rc<RefCell<bool>>,
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
        if *self.commit_attempts.borrow() >= self.commit_max_attemts {
            let message_id = format!("{}:{}:{}", topic, self.partition, self.offset);
            warn!(
                self.logger, "Stuck trying to commit processed task";
                "id" => &message_id
            );
            THREAD_TASK_CLEAR.with(|retry| *retry.borrow_mut() = true);
            return Err(ErrorKind::CommitRetryStuck(message_id).into());
        }
        *self.commit_attempts.borrow_mut() += 1;
        let mut list = TopicPartitionList::new();
        list.add_partition_offset(topic, self.partition, Offset(self.offset + 1));
        self.consumer
            .commit(&list, CommitMode::Sync)
            .with_context(|_| ErrorKind::CommitFailed)?;
        Ok(())
    }

    /// Publish a new task to kafka on the given retry topic.
    ///
    /// Also used to send tasks to the skipped topic.
    fn retry<Q: TaskQueue>(&self, topic: &str, task: Task<Q>) -> Result<()> {
        let mut headers = OwnedHeaders::new_with_capacity(task.headers.len());
        for (key, value) in &task.headers {
            headers = headers.add(key, value);
        }
        let retry_value = (task.retry_count + 1).to_string();
        headers = headers.add(KAFKA_TASKS_RETRY_HEADER, &retry_value);
        headers = headers.add(KAFKA_TASKS_ID_HEADER, &task.id().to_string());
        let record: FutureRecord<(), [u8]> = FutureRecord::to(topic)
            .headers(headers)
            .payload(&task.message);
        self.retry_producer
            .send(record, i64::from(self.retry_timeout))
            .wait()
            .with_context(|_| ErrorKind::RetryEnqueueID(task.id().to_string()))?
            .map_err(|(error, _)| error)
            .with_context(|_| ErrorKind::RetryEnqueueID(task.id().to_string()))?;
        Ok(())
    }
}

impl<Q: TaskQueue> AckStrategy<Q> for KafkaAck {
    fn fail(&self, task: Task<Q>) -> Result<()> {
        let topic = task.queue.name();
        let retry_topic = topic_for_queue(&self.prefix, &topic, TopicRole::Retry);
        let topic = topic_for_queue(&self.prefix, &topic, TopicRole::Queue);
        if !*self.retry_published.borrow() {
            self.retry(&retry_topic, task)?;
            *self.retry_published.borrow_mut() = true;
        }
        self.commit(&topic)?;
        self.clear_cache();
        Ok(())
    }

    fn skip(&self, task: Task<Q>) -> Result<()> {
        let topic = task.queue.name();
        let skip_topic = topic_for_queue(&self.prefix, &topic, TopicRole::Skip);
        let topic = topic_for_queue(&self.prefix, &topic, TopicRole::Queue);
        if !*self.skipped_published.borrow() {
            self.retry(&skip_topic, task)?;
            *self.skipped_published.borrow_mut() = true;
        }
        self.commit(&topic)?;
        self.clear_cache();
        Ok(())
    }

    fn success(&self, task: Task<Q>) -> Result<()> {
        let topic = task.queue.name();
        let topic = topic_for_queue(&self.prefix, &topic, TopicRole::Queue);
        self.commit(&topic)?;
        self.clear_cache();
        Ok(())
    }
}

/// Store information about a task cached in a ThreadLocal for retry publishing.
struct RetryCache {
    attempts: u8,
    message: OwnedMessage,
    published: bool,
}

impl RetryCache {
    fn new(message: OwnedMessage) -> RetryCache {
        RetryCache {
            attempts: 0,
            message,
            published: false,
        }
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
    /// Counter of attempts to commit the task.
    commit_attempts: Rc<RefCell<u8>>,
    /// Configured maximum number of attempts to commit.
    commit_max_attemts: u8,
    /// Kafka consumer to commit offsets on.
    consumer: Arc<BaseStatsConsumer>,
    /// The logger instance to log to.
    logger: Logger,
    /// Kafka message headers.
    headers: HashMap<String, String>,
    /// Kafka message body.
    message: Vec<u8>,
    /// Task ID.
    id: TaskId,
    /// Kafka message offset on partition.
    offset: i64,
    /// Kafka message partition in topic.
    partition: i32,
    /// Prefix of kafka topics.
    prefix: String,
    /// Name of the queue the task was from.
    queue: String,
    /// Number of times the task was retried.
    retry_count: u8,
    /// Kafka producer to retry tasks with.
    retry_producer: Arc<FutureProducer<ClientStatsContext>>,
    /// The task was already republished to the retry topic.
    retry_published: Rc<RefCell<bool>>,
    /// Timeout for kafka to ack retried tasks.
    retry_timeout: u32,
    /// The task was already republished to the skipped topic.
    skipped_published: Rc<RefCell<bool>>,
}

impl TaskCache {
    fn task<Q: TaskQueue>(self) -> Result<Task<Q>> {
        let queue = match self.queue.parse::<Q>() {
            Ok(queue) => queue,
            Err(error) => {
                return Err(error)
                    .context(ErrorKind::QueueNameInvalid(self.queue))
                    .map_err(Error::from)
            }
        };
        Ok(Task {
            ack_strategy: Arc::new(KafkaAck {
                commit_attempts: Rc::clone(&self.commit_attempts),
                commit_max_attemts: self.commit_max_attemts,
                consumer: self.consumer,
                logger: self.logger.clone(),
                offset: self.offset,
                partition: self.partition,
                prefix: self.prefix,
                retry_producer: self.retry_producer,
                retry_published: Rc::clone(&self.retry_published),
                retry_timeout: self.retry_timeout,
                skipped_published: Rc::clone(&self.skipped_published),
            }),
            headers: self.headers,
            id: self.id,
            message: self.message,
            processed: false,
            queue,
            retry_count: self.retry_count,
        })
    }
}
