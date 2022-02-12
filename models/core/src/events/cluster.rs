use serde::Deserialize;
use serde::Serialize;

use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::cluster::discovery::ClusterDiscovery;
use crate::cluster::ClusterSettings;

/// Metadata attached to cluster status change events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterChanged {
    pub after: ClusterDiscovery,
    pub before: ClusterDiscovery,
    pub cluster_id: String,
}

/// Enumerates all possible cluster events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum ClusterEvent {
    /// Service discovery record for a cluster changed.
    #[serde(rename = "CLUSTER_CHANGED")]
    Changed(ClusterChanged),

    /// Service discovery found a new cluster.
    #[serde(rename = "CLUSTER_NEW")]
    New(ClusterDiscovery),

    /// A synthetic ClusterSettings record was created for a discovered cluster without it.
    #[serde(rename = "CLUSTER_SETTINGS_SYNTHETIC")]
    SettingsSynthetic(ClusterSettings),
}

impl ClusterEvent {
    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        let cluster_id = match self {
            ClusterEvent::Changed(change) => &change.cluster_id,
            ClusterEvent::New(discovery) => &discovery.cluster_id,
            ClusterEvent::SettingsSynthetic(settings) => &settings.cluster_id,
        };
        Some(cluster_id)
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            ClusterEvent::Changed(_) => "CLUSTER_CHANGED",
            ClusterEvent::New(_) => "CLUSTER_NEW",
            ClusterEvent::SettingsSynthetic(_) => "CLUSTER_SETTINGS_SYNTHETIC",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

/// Build `ClusterEvent`s, validating inputs.
pub struct ClusterEventBuilder {
    pub(super) builder: EventBuilder,
}

impl ClusterEventBuilder {
    /// Build a `ClusterEvent::Changed` event.
    pub fn changed(self, before: ClusterDiscovery, after: ClusterDiscovery) -> Event {
        let event = ClusterEvent::Changed(ClusterChanged {
            cluster_id: before.cluster_id.clone(),
            before,
            after,
        });
        let payload = Payload::Cluster(event);
        self.builder.finish(payload)
    }

    /// Build a `ClusterEvent::New` event.
    pub fn new_cluster(self, discovery: ClusterDiscovery) -> Event {
        let event = ClusterEvent::New(discovery);
        let payload = Payload::Cluster(event);
        self.builder.finish(payload)
    }

    /// Build a `ClusterEvent::SettingsSynthetic` event.
    pub fn synthetic_settings(self, settings: ClusterSettings) -> Event {
        let event = ClusterEvent::SettingsSynthetic(settings);
        let payload = Payload::Cluster(event);
        self.builder.finish(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::ClusterChanged;
    use super::ClusterEvent;
    use super::Event;
    use super::Payload;
    use crate::cluster::discovery::ClusterDiscovery;

    #[test]
    fn changed() {
        let after = ClusterDiscovery::new("test", vec!["http://agent:1234".into()]);
        let before = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder()
            .cluster()
            .changed(before.clone(), after.clone());
        let expected = Payload::Cluster(ClusterEvent::Changed(ClusterChanged {
            after,
            before,
            cluster_id: "test".into(),
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn new_cluster() {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder().cluster().new_cluster(discovery.clone());
        let expected = Payload::Cluster(ClusterEvent::New(discovery));
        assert_eq!(event.payload, expected);
    }
}
