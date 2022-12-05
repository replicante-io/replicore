use replisdk::core::models::platform::Platform;
use serde::Deserialize;
use serde::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::scope::EntityId;

/// Enumerates all possible platform events emitted by the system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum PlatformEvent {
    /// A [`Platform`] object was applied.
    ///
    /// This event is emitted even if the object already exists and was not changed.
    #[serde(rename = "PLATFORM_APPLY")]
    Apply(Platform),
}

impl PlatformEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            PlatformEvent::Apply(_) => "PLATFORM_APPLY",
        }
    }

    /// Identifier of the namespace.
    pub fn entity_id(&self) -> EntityId {
        let namespace = match self {
            PlatformEvent::Apply(platform) => &platform.ns_id,
        };
        EntityId::Namespace(namespace)
    }
}

/// Build `PlatformEvent`s, validating inputs.
pub struct PlatformEventBuilder {
    pub(super) builder: EventBuilder,
}

impl PlatformEventBuilder {
    /// Build a `PlatformEvent::Apply` event.
    pub fn apply(self, platform: Platform) -> Event {
        let event = PlatformEvent::Apply(platform);
        let payload = Payload::Platform(event);
        self.builder.finish(payload)
    }
}
