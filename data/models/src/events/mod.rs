//! System events models and attributes.
use chrono::DateTime;
use chrono::Utc;

use super::Agent;
use super::AgentStatus;
use super::ClusterDiscovery;


mod builder;

use self::builder::EventBuilder;


/// Metadata attached to agent status change events.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentStatusChange {
    pub cluster: String,
    pub host: String,
    pub after: AgentStatus,
    pub before: AgentStatus,
}


/// Enumerates all possible events emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum EventData {
    /// Emitted when an agent is detected to be down.
    #[serde(rename = "AGENT_DOWN")]
    AgentDown(AgentStatusChange),

    /// The status of an agent was determined for the first time.
    #[serde(rename = "AGENT_NEW")]
    AgentNew(Agent),

    /// Emitted when an agent was be down but is now detected as up.
    #[serde(rename = "AGENT_RECOVER")]
    AgentRecover(AgentStatusChange),

    /// Emitted when an agent was detected to be down but the reason may have changed.
    #[serde(rename = "AGENT_STILL_DOWN")]
    AgentStillDown(AgentStatusChange),

    /// The service discovery found a new cluster.
    #[serde(rename = "CLUSTER_NEW")]
    ClusterNew(ClusterDiscovery),

    /// Emitted when a datastore is detected to be down.
    #[serde(rename = "DATASTORE_DOWN")]
    DatastoreDown(AgentStatusChange),

    /// Emitted when a datastore was be down but is now detected as up.
    #[serde(rename = "DATASTORE_RECOVER")]
    DatastoreRecover(AgentStatusChange),

    /// Emitted when a datastore was detected to be down but the reason may have changed.
    #[serde(rename = "DATASTORE_STILL_DOWN")]
    DatastoreStillDown(AgentStatusChange),
}


/// Model an event that is emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Event {
    pub payload: EventData,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create an helper object to build `Event`s.
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }

    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster(&self) -> Option<&str> {
        match self.payload {
            EventData::AgentDown(ref data) => Some(&data.cluster),
            EventData::AgentNew(ref data) => Some(&data.cluster),
            EventData::AgentRecover(ref data) => Some(&data.cluster),
            EventData::AgentStillDown(ref data) => Some(&data.cluster),
            EventData::ClusterNew(ref data) => Some(&data.cluster),
            EventData::DatastoreDown(ref data) => Some(&data.cluster),
            EventData::DatastoreRecover(ref data) => Some(&data.cluster),
            EventData::DatastoreStillDown(ref data) => Some(&data.cluster),
        }
    }
}


#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::Utc;
    use serde_json;

    use super::super::ClusterDiscovery;
    use super::Event;

    #[test]
    fn from_json() {
        let payload = r#"{"payload":{"event":"CLUSTER_NEW","data":{"cluster":"test","nodes":[]}},"timestamp":"2014-07-08T09:10:11.012Z"}"#;
        let event: Event = serde_json::from_str(&payload).unwrap();
        let discovery = ClusterDiscovery::new("test", vec![]);
        let expected = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms_micro(9, 10, 11, 12000))
            .cluster().cluster_new(discovery);
        assert_eq!(event, expected);
    }

    #[test]
    fn to_json() {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
            .cluster().cluster_new(discovery);
        let payload = serde_json::to_string(&event).unwrap();
        let expected = r#"{"payload":{"event":"CLUSTER_NEW","data":{"cluster":"test","nodes":[]}},"timestamp":"2014-07-08T09:10:11Z"}"#;
        assert_eq!(payload, expected);
    }
}
