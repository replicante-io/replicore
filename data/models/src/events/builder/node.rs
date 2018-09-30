use super::super::super::Node;
use super::super::Event;
use super::super::EventBuilder;
use super::super::EventPayload;


/// Build `Event`s that belongs to the node family.
pub struct NodeBuilder {
    builder: EventBuilder,
}

impl NodeBuilder {
    /// Create a new node event builder.
    pub fn builder(builder: EventBuilder) -> NodeBuilder {
        NodeBuilder { builder }
    }

    /// Build an `EventPayload::NodeNew`.
    pub fn node_new(self, node: Node) -> Event {
        let data = EventPayload::NodeNew(node);
        self.builder.build(data)
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::DatastoreInfo as WireNode;
    use super::Node;
    use super::Event;
    use super::EventPayload;

    #[test]
    fn node_new() {
        let node = WireNode::new("cluster", "TestDB", "test", "1.2.3");
        let node = Node::new(node);
        let event = Event::builder().node().node_new(node.clone());
        let expected = EventPayload::NodeNew(node);
        assert_eq!(event.payload, expected);
    }
}
