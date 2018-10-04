use super::super::super::Node;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;
use super::super::NodeChanged;


/// Build `Event`s that belongs to the node family.
pub struct NodeBuilder {
    builder: EventBuilder,
}

impl NodeBuilder {
    /// Create a new node event builder.
    pub fn builder(builder: EventBuilder) -> NodeBuilder {
        NodeBuilder { builder }
    }

    /// Build an `EventPayload::NodeChanged` event.
    pub fn changed(self, before: Node, after: Node) -> Event {
        let data = EventPayload::NodeChanged(NodeChanged {
            cluster: before.cluster.clone(),
            host: before.name.clone(),
            before,
            after,
        });
        self.builder.build(data)
    }

    /// Build an `EventPayload::NodeNew` event.
    pub fn node_new(self, node: Node) -> Event {
        let data = EventPayload::NodeNew(node);
        self.builder.build(data)
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::DatastoreInfo as WireNode;
    use super::Event;
    use super::EventPayload;
    use super::Node;
    use super::NodeChanged;

    #[test]
    fn changed() {
        let before = WireNode::new("cluster", "TestDB", "test", "1.2.3");
        let before = Node::new(before);
        let after = WireNode::new("cluster", "TestDB", "test", "4.5.6");
        let after = Node::new(after);
        let event = Event::builder().node().changed(before.clone(), after.clone());
        let expected = EventPayload::NodeChanged(NodeChanged {
            cluster: before.cluster.clone(),
            host: before.name.clone(),
            before,
            after,
        });
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn node_new() {
        let node = WireNode::new("cluster", "TestDB", "test", "1.2.3");
        let node = Node::new(node);
        let event = Event::builder().node().node_new(node.clone());
        let expected = EventPayload::NodeNew(node);
        assert_eq!(event.payload, expected);
    }
}
