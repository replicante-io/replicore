//! System events models and attributes.
use chrono::DateTime;
use chrono::Utc;

use super::AgentInfo;
use super::AgentStatus;
use super::ClusterDiscovery;
use super::Node;
use super::Shard;


mod builder;

use self::builder::EventBuilder;


/// Metadata attached to agent new events.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentNew {
    pub cluster: String,
    pub host: String,
}


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
pub enum EventPayload {
    /// An agent was found to be down.
    #[serde(rename = "AGENT_DOWN")]
    AgentDown(AgentStatusChange),

    /// Information about an agent was collected for the first time.
    #[serde(rename = "AGENT_INFO_NEW")]
    AgentInfoNew(AgentInfo),

    /// An agent was discovered for the first time.
    #[serde(rename = "AGENT_NEW")]
    AgentNew(AgentNew),

    /// An agent was found to be up.
    #[serde(rename = "AGENT_UP")]
    AgentUp(AgentStatusChange),

    /// Service discovery found a new cluster.
    #[serde(rename = "CLUSTER_NEW")]
    ClusterNew(ClusterDiscovery),

    /// A datastore node was found to be down.
    #[serde(rename = "NODE_DOWN")]
    NodeDown(AgentStatusChange),

    /// A datastore node was found for the first time.
    #[serde(rename = "NODE_NEW")]
    NodeNew(Node),

    /// A datastore node was found to be up.
    #[serde(rename = "NODE_UP")]
    NodeUp(AgentStatusChange),

    /// A shard was found for the first time on a node.
    #[serde(rename = "SHARD_ALLOCATION_NEW")]
    ShardAllocationNew(Shard),
}


/// Model an event that is emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub payload: EventPayload,
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
            EventPayload::AgentDown(ref data) => Some(&data.cluster),
            EventPayload::AgentInfoNew(ref data) => Some(&data.cluster),
            EventPayload::AgentNew(ref data) => Some(&data.cluster),
            EventPayload::AgentUp(ref data) => Some(&data.cluster),
            EventPayload::ClusterNew(ref data) => Some(&data.cluster),
            EventPayload::NodeDown(ref data) => Some(&data.cluster),
            EventPayload::NodeNew(ref data) => Some(&data.cluster),
            EventPayload::NodeUp(ref data) => Some(&data.cluster),
            EventPayload::ShardAllocationNew(ref data) => Some(&data.cluster),
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
        let payload = r#"{"event":"CLUSTER_NEW","data":{"cluster":"test","nodes":[]},"timestamp":"2014-07-08T09:10:11.012Z"}"#;
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
        let expected = r#"{"event":"CLUSTER_NEW","data":{"cluster":"test","nodes":[]},"timestamp":"2014-07-08T09:10:11Z"}"#;
        assert_eq!(payload, expected);
    }
}
