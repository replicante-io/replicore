use std::sync::Arc;

use humthreads::ThreadScope;
use opentracingrust::Tracer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::Logger;

use replicante_service_healthcheck::HealthChecks;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use crate::backend;
use crate::metrics::EMIT_ERROR;
use crate::metrics::EMIT_TOTAL;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::Iter;
use crate::Result;
use crate::StreamConfig;

/// Generic stream provider.
///
/// Provides a standard implementation for generating messages of any
/// `Serialize`able and `Deserialize`able types over a configurable backend.
///
/// This allows different streams of types to be defined but remain consistent
/// both for application developers, operators and end users.
///
/// This also means that a feature implemented for one stream is implemented
/// for all, avoiding messy interfaces, confusion and unpleasant user experiences.
#[derive(Clone)]
pub struct Stream<T>
where
    T: DeserializeOwned + Serialize + 'static,
{
    inner: Arc<dyn StreamInterface<T>>,
    logger: Logger,
    stream_id: &'static str,
    tracer: Option<Arc<Tracer>>,
}

impl<T> Stream<T>
where
    T: DeserializeOwned + Serialize + 'static,
{
    pub fn new(config: StreamConfig, opts: StreamOpts) -> Result<Stream<T>> {
        let logger = opts.logger.clone();
        let stream_id = opts.stream_id;
        let tracer = opts.tracer.clone();
        let backend = match config {
            StreamConfig::Kafka(config) => backend::kafka(config, opts)?,
        };
        Ok(Stream::with_backend(stream_id, backend, logger, tracer))
    }

    pub(crate) fn with_backend<TR>(
        stream_id: &'static str,
        inner: Arc<dyn StreamInterface<T>>,
        logger: Logger,
        tracer: TR,
    ) -> Stream<T>
    where
        TR: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Stream {
            inner,
            logger,
            stream_id,
            tracer,
        }
    }

    /// Emit a message to the stream.
    ///
    /// Messages are assigned to stream partitions based on the provided `id`.
    ///
    /// Messages with the same `id` are guaranteed to be delivered in the same order
    /// they are emitted.
    /// Messages with different `id`s have no ordering guarantees relative to each other.
    pub fn emit(&self, mut message: EmitMessage<T>) -> Result<()> {
        EMIT_TOTAL.with_label_values(&[self.stream_id]).inc();
        if let Err(error) = message.trace_inject(self.tracer.as_ref()) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context while emiting stream message";
                failure_info(&error),
            );
        }
        self.inner.emit(message).map_err(|error| {
            EMIT_ERROR.with_label_values(&[self.stream_id]).inc();
            error
        })
    }

    /// Return an `Iterator` that will fetch new messages from the stream.
    ///
    /// Once the iterator reaches the end of the stream it will block waiting for
    /// new messages to be emitted onto the stream.
    ///
    /// The `Iterator` will belong to the group of followers identified by `group`.
    /// Followers in the same group receive a partition of the overall stream.
    ///
    /// The `Iterator` will return messages starting from the last acknowledged
    /// message processed for this `group`.
    /// If no message was ever processed for the `group` the `Iterator` will start
    /// following messages from the oldest available message.
    ///
    /// # Shutdown and activity report
    /// In Replicante Core, followers are expected to run a background threads.
    ///
    /// To allow for a clean shutdown, Replicante uses a threads wrapper library that provides
    /// a `ThreadScope` object that can be used to check if the system is shutting down
    /// as well as to report the current thread activity.
    ///
    /// Followers must be passed a reference to the `ThreadScope` so that backends and
    /// backoff can check if the user requested a shutdown as well as report process.
    pub fn follow<'a, S, TS>(&self, group: S, thread: TS) -> Result<Iter<'a, T>>
    where
        S: Into<String>,
        TS: Into<Option<&'a ThreadScope>>,
    {
        self.inner.follow(group.into(), thread.into(), true)
    }

    /// Similar to `Stream::follow` but stops iterating at the end instead of blocking.
    pub fn short_follow<'a, S, TS>(&self, group: S, thread: TS) -> Result<Iter<'a, T>>
    where
        S: Into<String>,
        TS: Into<Option<&'a ThreadScope>>,
    {
        self.inner.follow(group.into(), thread.into(), false)
    }
}

/// Stream programmatic options, those that should not be user configuration
pub struct StreamOpts<'a> {
    pub(crate) healthchecks: &'a mut HealthChecks,
    pub(crate) logger: Logger,
    pub(crate) stream_id: &'static str,
    pub(crate) tracer: Option<Arc<Tracer>>,
}

impl<'a> StreamOpts<'a> {
    pub fn new<T>(
        stream_id: &'static str,
        healthchecks: &'a mut HealthChecks,
        logger: Logger,
        tracer: T,
    ) -> StreamOpts<'a>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        StreamOpts {
            healthchecks,
            logger,
            stream_id,
            tracer,
        }
    }
}
