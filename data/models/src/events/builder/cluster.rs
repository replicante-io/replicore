use super::super::super::ClusterDiscovery;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;


/// Build `Event`s that belongs to the cluster family.
pub struct ClusterBuilder {
    builder: EventBuilder,
}

impl ClusterBuilder {
    /// Create a new cluster event builder.
    pub fn builder(builder: EventBuilder) -> ClusterBuilder {
        ClusterBuilder { builder }
    }

    /// Build an `EventPayload::ClusterNew`.
    pub fn cluster_new(self, discovery: ClusterDiscovery) -> Event {
        let data = EventPayload::ClusterNew(discovery);
        self.builder.build(data)
    }
}


#[cfg(test)]
mod tests {
    use super::ClusterDiscovery;
    use super::Event;
    use super::EventPayload;

    #[test]
    fn cluster_new() {
        let discovery = ClusterDiscovery::new("test", vec![]);
        let event = Event::builder().cluster().cluster_new(discovery.clone());
        let expected = EventPayload::ClusterNew(discovery);
        assert_eq!(event.payload, expected);
    }
}
