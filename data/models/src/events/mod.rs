use chrono::DateTime;
use chrono::Utc;

use super::Agent;
use super::ClusterDiscovery;


mod builder;

use self::builder::EventBuilder;


/// Enumerates all possible events emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", deny_unknown_fields)]
pub enum EventData {
    /// The status of an agent was determined for the first time.
    #[serde(rename = "AGENT_NEW")]
    AgentNew(Agent),

    /// The service discovery found a new cluster.
    #[serde(rename = "CLUSTER_NEW")]
    ClusterNew(ClusterDiscovery),
}


/// Model an event that is emitted by the system.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event: EventData,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create an helper object to build `Event`s.
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
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
        let payload = r#"{"event":{"event":"CLUSTER_NEW","payload":{"name":"test","nodes":[]}},"timestamp":"2014-07-08T09:10:11.012Z"}"#;
        let event: Event = serde_json::from_str(&payload).unwrap();
        let discovery = ClusterDiscovery::new("test", vec![]);
        let expected = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms_micro(9, 10, 11, 12000))
            .cluster().new(discovery);
        assert_eq!(event, expected);
    }

    #[test]
    fn to_json() {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder()
            .timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
            .cluster().new(discovery);
        let payload = serde_json::to_string(&event).unwrap();
        let expected = r#"{"event":{"event":"CLUSTER_NEW","payload":{"name":"test","nodes":[]}},"timestamp":"2014-07-08T09:10:11Z"}"#;
        assert_eq!(payload, expected);
    }
}
