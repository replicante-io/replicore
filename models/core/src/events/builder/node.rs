use crate::events::Event;
use crate::events::EventBuilder;
use crate::events::EventPayload;
use crate::events::NodeChanged;
use crate::Node;

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
        let cluster_id = before.cluster_id.clone();
        let node_id = before.node_id.clone();
        let data = EventPayload::NodeChanged(Box::new(NodeChanged {
            after,
            before,
            cluster_id,
            node_id,
        }));
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
    use replicante_models_agent::info::DatastoreInfo as WireNode;

    use super::Event;
    use super::EventPayload;
    use super::Node;
    use super::NodeChanged;

    #[test]
    fn changed() {
        let before = WireNode::new("cluster", "TestDB", "test", "1.2.3", None);
        let before = Node::new(before);
        let after = WireNode::new("cluster", "TestDB", "test", "4.5.6", None);
        let after = Node::new(after);
        let event = Event::builder()
            .node()
            .changed(before.clone(), after.clone());
        let expected = EventPayload::NodeChanged(Box::new(NodeChanged {
            after,
            cluster_id: before.cluster_id.clone(),
            node_id: before.node_id.clone(),
            before,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn node_new() {
        let node = WireNode::new("cluster", "TestDB", "test", "1.2.3", None);
        let node = Node::new(node);
        let event = Event::builder().node().node_new(node.clone());
        let expected = EventPayload::NodeNew(node);
        assert_eq!(event.payload, expected);
    }
}
