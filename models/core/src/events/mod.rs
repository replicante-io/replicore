use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

pub mod action;
pub mod agent;
pub mod cluster;
pub mod discovery_settings;
pub mod node;
pub mod shard;

use crate::scope::EntityId;

/// Attempt to deserialize an event or return its code if desertification fails.
///
/// # Example
/// ```rust
/// use replicante_models_core::events::DeserializeResult;
/// use replicante_models_core::deserialize_event;
///
/// let encoded = concat!(
///     r#"{"category":"TEST","event":"TEST_NEW","payload":1,"#,
///     r#""timestamp":"2014-07-08T09:10:11.012Z"}"#
/// );
/// match deserialize_event!(serde_json::from_str, &encoded) {
///     DeserializeResult::Ok(event) => println!("Event: {:?}", event),
///     DeserializeResult::Unknown(code, _) => println!("unknown event code {:?}", code),
///     DeserializeResult::Err(error) => println!("{:?}", error),
/// };
/// ```
#[macro_export]
macro_rules! deserialize_event {
    ($decoder:path, $source:expr) => {
        match $decoder($source) {
            Ok(event) => $crate::events::DeserializeResult::Ok(event),
            Err(error) => match $decoder($source) {
                Ok(code) => $crate::events::DeserializeResult::Unknown(code, error),
                Err(_) => $crate::events::DeserializeResult::Err(error),
            },
        }
    };
}

/// Model an event that is emitted by the system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub payload: Payload,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create an helper object to build `Event`s.
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }

    /// Return the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match &self.payload {
            Payload::Action(event) => event.code(),
            Payload::Agent(event) => event.code(),
            Payload::Cluster(event) => event.code(),
            Payload::DiscoverySettings(event) => event.code(),
            Payload::Node(event) => event.code(),
            Payload::Shard(event) => event.code(),
            #[cfg(test)]
            Payload::Test(event) => event.code(),
        }
    }

    /// Identifier of the entity the event is about.
    pub fn entity_id(&self) -> EntityId {
        match &self.payload {
            Payload::Action(event) => event.entity_id(),
            Payload::Agent(event) => event.entity_id(),
            Payload::Cluster(event) => event.entity_id(),
            Payload::DiscoverySettings(event) => event.entity_id(),
            Payload::Node(event) => event.entity_id(),
            Payload::Shard(event) => event.entity_id(),
            #[cfg(test)]
            Payload::Test(event) => event.entity_id(),
        }
    }
}

/// Build `Event`s, validating inputs.
#[derive(Default)]
pub struct EventBuilder {
    timestamp: Option<DateTime<Utc>>,
}

impl EventBuilder {
    /// Create an empty `Event` builder.
    pub fn new() -> EventBuilder {
        EventBuilder::default()
    }

    /// Build action events.
    pub fn action(self) -> self::action::ActionEventBuilder {
        self::action::ActionEventBuilder { builder: self }
    }

    /// Build agent events.
    pub fn agent(self) -> self::agent::AgentEventBuilder {
        self::agent::AgentEventBuilder { builder: self }
    }

    /// Build cluster events.
    pub fn cluster(self) -> self::cluster::ClusterEventBuilder {
        self::cluster::ClusterEventBuilder { builder: self }
    }

    /// Build discovery settings events.
    pub fn discovery_settings(self) -> self::discovery_settings::DiscoverySettingsEventBuilder {
        self::discovery_settings::DiscoverySettingsEventBuilder { builder: self }
    }

    /// Build node events.
    pub fn node(self) -> self::node::NodeEventBuilder {
        self::node::NodeEventBuilder { builder: self }
    }

    /// Build shard events.
    pub fn shard(self) -> self::shard::ShardEventBuilder {
        self::shard::ShardEventBuilder { builder: self }
    }

    /// Set the event occurrence timestamp.
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Internal helper to build an `Event` out of a `Payload`.
    fn finish(self, payload: Payload) -> Event {
        Event {
            payload,
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
        }
    }
}

/// Event identification codes.
///
/// Useful to deal with unknown events, usually emitted from newer/older version of Replicante.
/// At a minimum it can be used to provide more informative error messages.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct EventCode {
    pub category: String,
    pub event: String,
}

/// Result of a `deserialize_event` operation.
#[allow(clippy::large_enum_variant)]
pub enum DeserializeResult<E> {
    /// The event was deserialized correctly.
    Ok(Event),

    /// The event failed to deserialize but identification codes were extracted.
    Unknown(EventCode, E),

    /// The event failed to deserialize and identification codes could not be expected.
    Err(E),
}

/// Enumerates all possible events emitted by the system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "category")]
// TODO: use when possible #[non_exhaustive]
pub enum Payload {
    /// Action related events.
    #[serde(rename = "ACTION")]
    Action(self::action::ActionEvent),

    /// Agent related events.
    #[serde(rename = "AGENT")]
    Agent(self::agent::AgentEvent),

    /// Cluster related events.
    #[serde(rename = "CLUSTER")]
    Cluster(self::cluster::ClusterEvent),

    #[serde(rename = "DISCOVERY_SETTINGS")]
    DiscoverySettings(self::discovery_settings::DiscoverySettingsEvent),

    /// Node related events.
    #[serde(rename = "NODE")]
    Node(self::node::NodeEvent),

    /// Shard related events.
    #[serde(rename = "SHARD")]
    Shard(self::shard::ShardEvent),

    /// Events variant used exclusively for crate tests.
    #[cfg(test)]
    #[serde(rename = "TEST")]
    Test(TestEvent),
}

/// Events enum used exclusively for crate tests.
#[cfg(test)]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum TestEvent {
    #[serde(rename = "TEST_NEW")]
    New(i64),
}

#[cfg(test)]
impl TestEvent {
    pub fn cluster_id(&self) -> Option<&str> {
        None
    }

    pub fn code(&self) -> &'static str {
        match self {
            TestEvent::New(_) => "TEST_NEW",
        }
    }

    /// Identifier of the entity the test event is about.
    pub fn entity_id(&self) -> EntityId {
        EntityId::System
    }

    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::Utc;

    use super::DeserializeResult;
    use super::Event;
    use super::EventCode;
    use super::Payload;
    use super::TestEvent;
    use crate::deserialize_event;

    #[test]
    fn flatten_as_expected() {
        let event = Event {
            payload: Payload::Test(TestEvent::New(1)),
            timestamp: Utc.ymd(2014, 7, 8).and_hms_micro(9, 10, 11, 12000),
        };
        let actual = serde_json::to_string(&event).unwrap();
        let expected = concat!(
            r#"{"category":"TEST","event":"TEST_NEW","payload":1,"#,
            r#""timestamp":"2014-07-08T09:10:11.012Z"}"#
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_err() {
        let encoded = r#"{"event":"TEST_UNKOWN","payload":1}"#;
        let actual = match deserialize_event!(serde_json::from_str, &encoded) {
            DeserializeResult::Ok(_) => panic!("event decoding should fail"),
            DeserializeResult::Unknown(_, _) => panic!("unknown event code"),
            DeserializeResult::Err(error) => error,
        };
        let actual = format!("{:?}", actual);
        let expected = r#"Error("missing field `timestamp`", line: 1, column: 35)"#.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_ok() {
        let encoded = concat!(
            r#"{"category":"TEST","event":"TEST_NEW","payload":1,"#,
            r#""timestamp":"2014-07-08T09:10:11.012Z"}"#
        );
        let actual = match deserialize_event!(serde_json::from_str, &encoded) {
            DeserializeResult::Ok(expected) => expected,
            DeserializeResult::Unknown(_, _) => panic!("unknown event code"),
            DeserializeResult::Err(error) => panic!("{:?}", error),
        };
        let expected = Event {
            payload: Payload::Test(TestEvent::New(1)),
            timestamp: Utc.ymd(2014, 7, 8).and_hms_micro(9, 10, 11, 12000),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_unkown() {
        let encoded = concat!(
            r#"{"category":"TEST","event":"TEST_UNKOWN","payload":1,"#,
            r#""timestamp":"2014-07-08T09:10:11.012Z"}"#
        );
        let actual = match deserialize_event!(serde_json::from_str, &encoded) {
            DeserializeResult::Ok(_) => panic!("event decoding should fail"),
            DeserializeResult::Unknown(expected, _) => expected,
            DeserializeResult::Err(error) => panic!("{:?}", error),
        };
        let expected = EventCode {
            category: "TEST".into(),
            event: "TEST_UNKOWN".into(),
        };
        assert_eq!(actual, expected);
    }
}
