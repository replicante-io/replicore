use bson::UtcDateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::Event;
use replicante_models_core::EventPayload;

/// Wrap an `Event` to allow BSON to encode/decode timestamps correctly.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(flatten)]
    pub payload: EventPayload,
    pub timestamp: UtcDateTime,
}

impl From<Event> for EventDocument {
    fn from(event: Event) -> EventDocument {
        EventDocument {
            payload: event.payload,
            timestamp: UtcDateTime(event.timestamp),
        }
    }
}

impl From<EventDocument> for Event {
    fn from(event: EventDocument) -> Event {
        Event {
            payload: event.payload,
            timestamp: event.timestamp.0,
        }
    }
}