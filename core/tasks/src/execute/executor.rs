//! Tasks Executor implementation.
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use opentelemetry_api::trace::FutureExt;
use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::trace::Tracer;
use opentelemetry_api::Context as OTelContext;

use replisdk::core::models::auth::Action;
use replisdk::core::models::auth::AuthContext;
use replisdk::core::models::auth::Resource;
use replisdk::utils::trace::TraceFutureErrExt;

use replicore_context::Context;

use super::backoff::Backoff;
use super::ReceivedTask;
use super::TaskAck;
use super::TaskCallback;
use super::TaskSource;
use crate::conf::Queue;
use crate::conf::TasksExecutorConf;
use crate::error::AbandonTask;

/// Asynchronously execute subscribed tasks when they become available.
///
/// The executor is initialised with a queue backend and subscribed to queue for tasks to process.
/// When tasks become available they are spawned for background executor.
/// Execution tasks are tracked to [`ack`]/[`nack`] and to terminate them on process exit.
///
/// ## Execution Pool and Capacity
///
/// Tasks are executed in parallel using [`tokio::spawn`].
///
/// To prevent a large number of tasks from overloading the system there is a
/// configurable limit to the number of tasks executed concurrently.
/// When this limit is reached the [`TasksExecutor`] will stop asking for new tasks.
///
/// ## Executor Shutdown
///
/// A process shutdown notification can be received by resolving a unit [`Future`].
/// If the exit signal future resolves the executor will:
///
/// 1. Stop fetching new tasks to execute.
/// 2. Abandon execution of all in-progress tasks (the standard retry logic will apply here).
///
/// ## Error Handling
///
/// The [`TasksExecutor`] has two possible sources of errors:
///
/// 1. Interactions with the Message Queue Platform (to fetch or acknowledge tasks).
/// 2. The task handlers executed when tasks are received.
///
/// ### Message Queue Errors
///
/// These errors are often indication of unavailability of the configured Message Queue Platform.
/// As availability issues tend to be transient interaction with the Message Queue is retried.
///
/// Retries are performed with an increasing delay up to a maximum threshold.
/// If the Message Queue Platform does not recover after a configurable number of attempts
/// the process terminates.
///
/// ### Errors during task execution
///
/// Task execution errors include application or configuration errors among others.
/// How recoverable these errors are greatly depends on the situation.
///
/// Message Queues have builtin systems to attempt redelivery of messages (tasks) in case
/// processes working on them fail.
/// The retry configuration depends on the queue the task is on.
///
/// When a task handler encounters a permanent errors were retries won't help,
/// it can mark the error with the [`AbandonTask`] context.
/// This causes the [`TasksExecutor`] to ack the task as completed and avoids needless retries.
///
/// [`ack`]: TaskSourceBackend::ack
/// [`nack`]: TaskSourceBackend::nack
pub struct TasksExecutor {
    ack: TaskAck,
    callbacks: HashMap<String, Arc<dyn TaskCallback>>,
    conf: TasksExecutorConf,
    pool: FuturesUnordered<tokio::task::JoinHandle<Result<()>>>,
    source: TaskSource,
}

impl TasksExecutor {
    /// Initialise a new task executor waiting for tasks from a [`TaskSourceBackend`].
    pub fn new(source: TaskSource, ack: TaskAck, conf: TasksExecutorConf) -> TasksExecutor {
        TasksExecutor {
            ack,
            callbacks: Default::default(),
            conf,
            pool: FuturesUnordered::new(),
            source,
        }
    }

    /// Execute tasks once they are received.
    pub async fn execute(
        &mut self,
        context: &Context,
        exit: impl Future<Output = ()>,
    ) -> Result<()> {
        let mut propagate_panic = None;
        let dispatch = self
            .execute_inner(context, exit, &mut propagate_panic)
            .await;

        // If the loop stopped we either failed or the process is exiting.
        // In either case proceed to cancel executing tokio tasks before we return.
        for task in self.pool.iter() {
            task.abort();
        }
        self.pool.clear();

        // Propagate panics (if any) or exit.
        if let Some(payload) = propagate_panic {
            slog::error!(
                context.logger,
                "Propagating panic from async task execution"
            );
            std::panic::resume_unwind(payload);
        }
        dispatch
    }

    /// Register a callback to execute tasks received on the corresponding queue.
    pub async fn subscribe<C>(
        &mut self,
        context: &Context,
        queue: &'static Queue,
        callback: C,
    ) -> Result<()>
    where
        C: TaskCallback + 'static,
    {
        let callback = Arc::new(callback);
        self.subscribe_arc(context, queue, callback).await
    }

    /// Implement the fetch, dispatch, join loop for queue processing.
    async fn execute_inner(
        &mut self,
        context: &Context,
        exit: impl Future<Output = ()>,
        propagate_panic: &mut Option<Box<dyn Any + Send + 'static>>,
    ) -> Result<()> {
        // Pin the exit future so we can select it across loops.
        tokio::pin!(exit);

        // Track errors while processing async task.
        let mut ack_backoff = Backoff::new(&self.conf.backoff);
        let mut source_backoff = Backoff::new(&self.conf.backoff);

        // Process tasks as they come in, until exit or error.
        loop {
            let poll_task = self.pool.len() < self.conf.concurrent_tasks;
            tokio::select! {
                // Exit early if process needs to shut down.
                _ = &mut exit => break,

                // Wait for async tasks to execute.
                task = self.source.next(context), if poll_task => {
                    let task = match task {
                        Err(error) => {
                            source_backoff.retry(context, error).await?;
                            continue;
                        }
                        Ok(task) => {
                            source_backoff.success();
                            task
                        }
                    };
                    self.execute_task(context, task).await;
                },

                // Report on results from async tasks execution.
                result = self.pool.next(), if !self.pool.is_empty() => {
                    let result = match result {
                        None => continue,
                        Some(result) => result,
                    };
                    match result {
                        Err(error) if error.is_panic() => {
                            *propagate_panic = Some(error.into_panic());
                            break;
                        }
                        Err(error) if error.is_cancelled() => slog::debug!(
                            context.logger, "Ignoring cancelled execution of async task"
                        ),
                        Err(error) => {
                            let error = anyhow::Error::from(error);
                            slog::warn!(
                                context.logger, "Unknown error from async task";
                                replisdk::utils::error::slog::ErrorAttributes::from(&error),
                            );
                        }
                        Ok(Err(error)) => ack_backoff.retry(context, error).await?,
                        Ok(Ok(())) => ack_backoff.success(),
                    };
                },
            };
        }

        // Task processing stopped for non-error reasons.
        Ok(())
    }

    /// Dispatch execution of a received task and handle context and acknowledgment.
    async fn execute_task(&self, context: &Context, mut task: ReceivedTask) {
        // Find the callback to handle the received task.
        let handler = self
            .callbacks
            .get(&task.queue.queue)
            .expect("received message for a queue we are not subscribed to")
            .clone();

        // Derive an updated context to propagate task specific information.
        let mut context = context
            .derive()
            .log_values(slog::o!("task_id" => task.id.clone()));
        if let Some(run_as) = task.run_as.take() {
            let auth = AuthContext {
                action: Action::define("task", "execute"),
                entity: run_as.entity,
                impersonate: run_as.impersonate,
                resource: Resource {
                    kind: String::from("Task"),
                    metadata: Default::default(),
                    resource_id: task.id.clone(),
                },
            };
            context = context.authenticated(auth);
        }
        let context = context.build();

        // Extract available tracing context and create a new span for the task to execute as.
        let otel_parent = task.trace.take().unwrap_or_else(OTelContext::current);
        let mut span = crate::telemetry::TRACER.span_builder("task.execute");
        span.span_kind = Some(opentelemetry_api::trace::SpanKind::Consumer);
        let span = crate::telemetry::TRACER.build_with_context(span, &otel_parent);
        let otel_context = otel_parent.with_span(span);

        // Execute the task in parallel with other activities.
        let ack_backend = self.ack.clone();
        let work = async move {
            let ack_backend = ack_backend;
            let result = handler.execute(&context, &task).await;
            match result {
                Ok(()) => ack_backend.done(&context, &task).await,
                Err(error) => {
                    slog::warn!(
                        context.logger, "Background Task encountered an error during processing";
                        replisdk::utils::error::slog::ErrorAttributes::from(&error),
                    );
                    let abandon = error.is::<AbandonTask>()
                        || error.chain().any(|cause| cause.is::<AbandonTask>());
                    if abandon {
                        ack_backend.done(&context, &task).await?;
                    }
                    Ok(())
                }
            }
        };
        let work = work.trace_on_err_with_status().with_context(otel_context);
        let join = tokio::spawn(work);
        self.pool.push(join);
    }

    /// Register a callback to execute tasks received on the corresponding queue.
    async fn subscribe_arc(
        &mut self,
        context: &Context,
        queue: &'static Queue,
        callback: Arc<dyn TaskCallback>,
    ) -> Result<()> {
        // Skip subscription if filters tell us to ignore the queue.
        let filters = &self.conf.filters;
        if filters.ignore.contains(&queue.queue) {
            return Ok(());
        }
        if !filters.process.is_empty() && !filters.process.contains(&queue.queue) {
            return Ok(());
        }

        // Fail if the queue is already subscribed.
        if self.callbacks.contains_key(&queue.queue) {
            anyhow::bail!(crate::error::AlreadySubscribed::new(&queue.queue));
        }

        // Register the queue handler and subscribe to tasks.
        slog::info!(
            context.logger,
            "Subscribed to queue '{}' for task execution", queue.queue;
            "queue" => &queue.queue
        );
        self.callbacks.insert(queue.queue.clone(), callback);
        self.source.subscribe(context, queue).await
    }
}

/// Incrementally build a [`TasksExecutor`] instance.
pub struct TasksExecutorBuilder {
    callbacks: Vec<(&'static Queue, Arc<dyn TaskCallback>)>,
    conf: TasksExecutorConf,
}

impl TasksExecutorBuilder {
    /// Initialise incremental configuration for a [`TasksExecutor`].
    pub fn new(conf: TasksExecutorConf) -> TasksExecutorBuilder {
        TasksExecutorBuilder {
            callbacks: Default::default(),
            conf,
        }
    }

    /// Complete this configuration with a [`TaskSourceBackend`] and return a [`TasksExecutor`].
    pub async fn build(
        self,
        context: &Context,
        source: TaskSource,
        ack: TaskAck,
    ) -> Result<TasksExecutor> {
        let mut tasks = TasksExecutor::new(source, ack, self.conf);
        for (queue, callback) in self.callbacks.into_iter() {
            tasks.subscribe_arc(context, queue, callback).await?;
        }
        Ok(tasks)
    }

    /// Register a callback to execute tasks received on the corresponding queue.
    pub fn subscribe<C>(&mut self, queue: &'static Queue, callback: C)
    where
        C: TaskCallback + 'static,
    {
        let callback: Arc<dyn TaskCallback> = Arc::new(callback);
        let info = (queue, callback);
        self.callbacks.push(info);
    }
}
