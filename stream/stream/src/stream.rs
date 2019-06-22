use std::sync::Arc;

use humthreads::ThreadScope;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::metrics::EMIT_ERROR;
use crate::metrics::EMIT_TOTAL;
use crate::traits::StreamInterface;
use crate::EmitMessage;
use crate::Iter;
use crate::Result;

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
    stream_id: &'static str,
}

impl<T> Stream<T>
where
    T: DeserializeOwned + Serialize + 'static,
{
    // TODO: remove once the first real backend uses this.
    #[allow(dead_code)]
    pub(crate) fn with_backend(
        stream_id: &'static str,
        inner: Arc<dyn StreamInterface<T>>,
    ) -> Stream<T> {
        Stream { inner, stream_id }
    }

    /// Emit a message to the stream.
    ///
    /// Messages are assigned to stream partitions based on the provided `id`.
    ///
    /// Messages with the same `id` are guaranteed to be delivered in the same order
    /// they are emitted.
    /// Messages with different `id`s have no ordering guarantees relative to each other.
    pub fn emit(&self, message: EmitMessage) -> Result<()> {
        EMIT_TOTAL.with_label_values(&[&self.stream_id]).inc();
        self.inner.emit(message).map_err(|error| {
            EMIT_ERROR.with_label_values(&[&self.stream_id]).inc();
            error
        })
    }

    /// Return an `Iterator` that will fetch new messages from the stream.
    ///
    /// Once the iterator reaches the end of the stream it will block waiting for
    /// new messages to be emitted onto the stream.
    /// # TODO: stop consuming on shutdown?
    /// # TODO: stop consuming on stream end?
    ///
    /// The `Iterator` will belong to the group of followers identified by `group`.
    /// Followers in the same group receive a partition of the overall stream.
    ///
    /// The `Iterator` will return messages starting from the last acknowledged
    /// message processed for this `group`.
    /// If no message was ever processed for the `group` the `Iterator` will start
    /// following messages from the oldest available message.
    pub fn follow<'a, S>(&self, group: S, thread: &'a ThreadScope) -> Result<Iter<'a, T>>
    where
        S: Into<String>,
    {
        self.inner.follow(group.into(), thread, true)
    }

    /// Similar to `Stream::follow` but stops iterating at the end instead of blocking.
    pub fn short_follow<'a, S>(&self, group: S, thread: &'a ThreadScope) -> Result<Iter<'a, T>>
    where
        S: Into<String>,
    {
        self.inner.follow(group.into(), thread, false)
    }
}
