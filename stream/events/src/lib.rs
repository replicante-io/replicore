use std::ops::Deref;
use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_models_core::events::Event;
use replicante_service_healthcheck::HealthChecks;
use replicante_stream::EmitMessage as BaseEmitMessage;
use replicante_stream::ErrorKind;
use replicante_stream::Iter as BaseIter;
use replicante_stream::Message as BaseMessage;
use replicante_stream::Result;
use replicante_stream::Stream as BaseStream;
use replicante_stream::StreamConfig;
use replicante_stream::StreamOpts;

const STREAM_ID: &str = "events";

/// `replicante_stream::EmitMessage` specialised to `Event`s
pub type EmitMessage = BaseEmitMessage<Event>;

/// `replicante_stream::Iter` specialised to `Event`s
pub type Iter<'a> = BaseIter<'a, Event>;

/// `replicante_stream::Message` specialised to `Event`s
pub type Message = BaseMessage<Event>;

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

    /// Extract the `Event::code` from the message.
    pub fn event_code(message: &Message) -> Result<String> {
        message
            .json_payload()?
            .get("event")
            .ok_or_else(|| ErrorKind::MessageInvalidAttribute("event", "attribute missing"))?
            .as_str()
            .ok_or_else(|| ErrorKind::MessageInvalidAttribute("event", "not a string").into())
            .map(|code| code.to_string())
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use replicante_models_core::cluster::ClusterDiscovery;
    use replicante_models_core::events::Event;
    use replicante_stream::test_support::mock_message;
    use replicante_stream::ErrorKind;

    use crate::Stream;

    #[test]
    fn event_code_ok() {
        let cluster = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder().cluster().cluster_new(cluster);
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        let event_code = Stream::event_code(&message).unwrap();
        assert_eq!(event_code, "CLUSTER_NEW");
    }

    #[test]
    fn event_code_missing() {
        let event = json!({});
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        match Stream::event_code(&message) {
            Ok(_) => panic!("should not have an event code"),
            Err(error) => match error.kind() {
                ErrorKind::MessageInvalidAttribute(_, reason) => {
                    assert_eq!(*reason, "attribute missing")
                }
                _ => panic!("unexpected error {:?}", error),
            },
        };
    }

    #[test]
    fn event_code_not_string() {
        let event = json!({"event": 42});
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        match Stream::event_code(&message) {
            Ok(_) => panic!("should not have an event code"),
            Err(error) => match error.kind() {
                ErrorKind::MessageInvalidAttribute(_, reason) => {
                    assert_eq!(*reason, "not a string")
                }
                _ => panic!("unexpected error {:?}", error),
            },
        };
    }
}
