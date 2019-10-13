use std::ops::Deref;
use std::sync::Arc;

use failure::Fail;
use opentracingrust::Tracer;
use slog::Logger;

use replicante_models_core::deserialize_event;
use replicante_models_core::events::DeserializeResult;
use replicante_models_core::events::Event;
use replicante_service_healthcheck::HealthChecks;
use replicante_stream::EmitMessage as BaseEmitMessage;
use replicante_stream::Error;
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

    /// Attemt to deserialize and event or return an event code if deserialization fails.
    pub fn deserialize_event(message: &Message) -> DeserializeResult<Error> {
        let payload = match message.json_payload() {
            Ok(json) => json,
            Err(error) => {
                let error = error.context(ErrorKind::PayloadDecode);
                return DeserializeResult::Err(error.into());
            }
        };
        // payload.clone is marked as redundant but the macro may use the value twice
        // in case it needs to be decoded a second time to look for the codes.
        #[allow(clippy::redundant_clone)]
        match deserialize_event!(serde_json::from_value, payload.clone()) {
            DeserializeResult::Ok(event) => DeserializeResult::Ok(event),
            DeserializeResult::Unknown(code, error) => {
                let error = error.context(ErrorKind::PayloadDecode);
                DeserializeResult::Unknown(code, error.into())
            }
            DeserializeResult::Err(error) => {
                let error = error.context(ErrorKind::PayloadDecode);
                DeserializeResult::Err(error.into())
            }
        }
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
    use replicante_models_core::events::DeserializeResult;
    use replicante_models_core::events::Event;
    use replicante_stream::test_support::mock_message;

    use crate::Stream;

    #[test]
    fn event_code_ok() {
        let cluster = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder().cluster().new_cluster(cluster);
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        match Stream::deserialize_event(&message) {
            DeserializeResult::Ok(_) => (),
            DeserializeResult::Unknown(_, _) => panic!("unknown event found"),
            DeserializeResult::Err(_) => panic!("failed to decode event"),
        };
    }

    #[test]
    fn event_code_missing() {
        let event = json!({});
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        match Stream::deserialize_event(&message) {
            DeserializeResult::Ok(_) => panic!("unexpected event found"),
            DeserializeResult::Unknown(_, _) => panic!("unknown event found"),
            DeserializeResult::Err(_) => (),
        };
    }

    #[test]
    fn event_code_not_string() {
        let event = json!({"event": 42});
        let message = mock_message("stream", "group", "id", HashMap::new(), event).unwrap();
        match Stream::deserialize_event(&message) {
            DeserializeResult::Ok(_) => panic!("unexpected event found"),
            DeserializeResult::Unknown(_, _) => panic!("unknown event found"),
            DeserializeResult::Err(_) => (),
        };
    }
}
