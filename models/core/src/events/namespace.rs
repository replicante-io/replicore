use serde_derive::Deserialize;
use serde_derive::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::cluster::discovery::DiscoverySettings;

/// Enumerates all possible agent events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum NamespaceEvent {
    /// A cluster DiscoverySettings object was added or updated.
    #[serde(rename = "NAMESPACE_DISCOVERY_SETTINGS")]
    DiscoverySettings(DiscoverySettings),
}

impl NamespaceEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            NamespaceEvent::DiscoverySettings(_) => "NAMESPACE_DISCOVERY_SETTINGS",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        match self {
            NamespaceEvent::DiscoverySettings(info) => &info.namespace,
        }
    }
}

/// Build `NamespaceEvent`s, validating inputs.
pub struct NamespaceEventBuilder {
    pub(super) builder: EventBuilder,
}

impl NamespaceEventBuilder {
    pub fn discovery_settings(self, settings: DiscoverySettings) -> Event {
        let event = NamespaceEvent::DiscoverySettings(settings);
        let payload = Payload::Namespace(event);
        self.builder.finish(payload)
    }
}
