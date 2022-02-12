use serde::Deserialize;
use serde::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::cluster::discovery::DiscoverySettings;

/// Enumerates all possible discovery settings events emitted by the system.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum DiscoverySettingsEvent {
    /// A DiscoverySettings object was applied.
    ///
    /// This event is emitted even if the object already exists and was not changed.
    #[serde(rename = "DISCOVERY_SETTINGS_APPLY")]
    Apply(DiscoverySettings),

    /// A DiscoverySettings object was deleted.
    ///
    /// This event is emitted even if no matching object exists.
    #[serde(rename = "DISCOVERY_SETTINGS_DELETE")]
    Delete(DiscoverySettingsDeleted),
}

impl DiscoverySettingsEvent {
    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            DiscoverySettingsEvent::Apply(_) => "DISCOVERY_SETTINGS_APPLY",
            DiscoverySettingsEvent::Delete(_) => "DISCOVERY_SETTINGS_DELETE",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        match self {
            DiscoverySettingsEvent::Apply(settings) => &settings.namespace,
            DiscoverySettingsEvent::Delete(id) => &id.namespace,
        }
    }
}

/// Build `DiscoverySettingsEventBuilder`s, validating inputs.
pub struct DiscoverySettingsEventBuilder {
    pub(super) builder: EventBuilder,
}

impl DiscoverySettingsEventBuilder {
    /// Build a `DiscoverySettingsEvent::Apply` event.
    pub fn apply(self, settings: DiscoverySettings) -> Event {
        let event = DiscoverySettingsEvent::Apply(settings);
        let payload = Payload::DiscoverySettings(event);
        self.builder.finish(payload)
    }

    /// Build a `DiscoverySettingsEvent::Delete` event.
    pub fn delete(self, namespace: String, name: String) -> Event {
        let id = DiscoverySettingsDeleted { namespace, name };
        let event = DiscoverySettingsEvent::Delete(id);
        let payload = Payload::DiscoverySettings(event);
        self.builder.finish(payload)
    }
}

/// Identification attributes of the DiscoverySettings object that was deleted.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverySettingsDeleted {
    pub namespace: String,
    pub name: String,
}
