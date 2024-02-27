use serde::Deserialize;
use serde::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::scope::EntityId;
use crate::scope::Namespace;

/// Enumerates all possible namespace events emitted by the system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum NamespaceEvent {
    /// A Namespace object was applied.
    ///
    /// This event is emitted even if the object already exists and was not changed.
    #[serde(rename = "NAMESPACE_APPLY")]
    Apply(Namespace),
}

impl NamespaceEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            NamespaceEvent::Apply(_) => "NAMESPACE_APPLY",
        }
    }

    /// Identifier of the namespace.
    pub fn entity_id(&self) -> EntityId {
        let namespace = match self {
            NamespaceEvent::Apply(ns) => &ns.ns_id,
        };
        EntityId::Namespace(namespace)
    }
}

/// Build `NamespaceEventBuilder`s, validating inputs.
pub struct NamespaceEventBuilder {
    pub(super) builder: EventBuilder,
}

impl NamespaceEventBuilder {
    /// Build a `NamespaceEventEvent::Apply` event.
    pub fn apply(self, namespace: Namespace) -> Event {
        let event = NamespaceEvent::Apply(namespace);
        let payload = Payload::Namespace(event);
        self.builder.finish(payload)
    }
}
