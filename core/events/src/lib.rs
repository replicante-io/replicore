//! Events platform interface for RepliCore Control Plane.
use std::collections::BTreeMap;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use time::OffsetDateTime;

pub mod emit;
mod errors;

pub use self::errors::Error;

/// An individual event emitted by the Control Plane.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Identifier of the specific event (and its payload type).
    pub code: String,

    /// Additional unstructured metadata attached to the event.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// JSON encoded event payload.
    #[serde(default)]
    pub payload: Value,

    /// Time the event was generated.
    #[serde(with = "time::serde::rfc3339")]
    pub time: OffsetDateTime,
}

impl Event {
    /// Attempt to decode the event payload into the specified type.
    pub fn decode<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.payload.clone())
            .context(Error::PayloadDecode)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn decode_event() {
        let event = Event {
            code: "TEST".into(),
            metadata: Default::default(),
            payload: serde_json::json!("test string"),
            time: time::OffsetDateTime::now_utc(),
        };
        let actual: String = event.decode().unwrap();
        assert_eq!(actual, "test string");
    }
}
