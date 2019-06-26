use std::ops::Deref;
use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_models_core::Event;
use replicante_service_healthcheck::HealthChecks;
use replicante_stream::EmitMessage as BaseEmitMessage;
use replicante_stream::Iter as BaseIter;
use replicante_stream::Result;
use replicante_stream::Stream as BaseStream;
use replicante_stream::StreamConfig;
use replicante_stream::StreamOpts;

const STREAM_ID: &str = "events";

/// `replicante_stream::EmitMessage` specialised to `Event`s
pub type EmitMessage = BaseEmitMessage<Event>;

/// `replicante_stream::Iter` specialised to `Event`s
pub type Iter<'a> = BaseIter<'a, Event>;

/// `replicante_stream::Stream` specialised to `Event`s
#[derive(Clone)]
pub struct Stream(BaseStream<Event>);

impl Stream {
    pub fn new<T>(
        config: StreamConfig,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Stream>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let opts = StreamOpts::new(STREAM_ID, healthchecks, logger, tracer);
        BaseStream::new(config, opts).map(Stream)
    }

    /// Return a `MockStream` version of the `Event`s stream for tests.
    #[cfg(feature = "with_test_support")]
    pub fn mock() -> Stream {
        let inner = replicante_stream::test_support::MockStream::make(STREAM_ID);
        Stream(inner)
    }
}

impl Deref for Stream {
    type Target = BaseStream<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
