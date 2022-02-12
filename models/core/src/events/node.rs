use serde::Deserialize;
use serde::Serialize;

use super::agent::StatusChange;
use super::Event;
use super::EventBuilder;
use super::Payload;
use crate::agent::Node;

/// Metadata attached to node changed events.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeChanged {
    pub after: Node,
    pub before: Node,
    pub cluster_id: String,
    pub node_id: String,
}

/// Enumerates all possible node events emitted by the system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
// TODO: use when possible #[non_exhaustive]
pub enum NodeEvent {
    /// A datastore node has changed.
    #[serde(rename = "NODE_CHANGED")]
    Changed(NodeChanged),

    /// A datastore node was found to be down.
    #[serde(rename = "NODE_DOWN")]
    Down(StatusChange),

    /// A datastore node was found for the first time.
    #[serde(rename = "NODE_NEW")]
    New(Node),

    /// A datastore node was found to be up.
    #[serde(rename = "NODE_UP")]
    Up(StatusChange),
}

impl NodeEvent {
    /// Look up the cluster ID for the event, if they have one.
    pub fn cluster_id(&self) -> Option<&str> {
        let cluster_id = match self {
            NodeEvent::Changed(change) => &change.cluster_id,
            NodeEvent::Down(change) => &change.cluster_id,
            NodeEvent::New(node) => &node.cluster_id,
            NodeEvent::Up(change) => &change.cluster_id,
        };
        Some(cluster_id)
    }

    /// Returns the event "code", the string that represents the event type.
    pub fn code(&self) -> &'static str {
        match self {
            NodeEvent::Changed(_) => "NODE_CHANGED",
            NodeEvent::Down(_) => "NODE_DOWN",
            NodeEvent::New(_) => "NODE_NEW",
            NodeEvent::Up(_) => "NODE_UP",
        }
    }

    /// Returns the "ordering ID" for correctly streaming the event.
    pub fn stream_key(&self) -> &str {
        self.cluster_id().unwrap_or("<system>")
    }
}

/// Build `NodeEvent`s, validating inputs.
pub struct NodeEventBuilder {
    pub(super) builder: EventBuilder,
}

impl NodeEventBuilder {
    /// Build a `NodeEvent::Changed` event.
    pub fn changed(self, before: Node, after: Node) -> Event {
        let cluster_id = before.cluster_id.clone();
        let node_id = before.node_id.clone();
        let event = NodeEvent::Changed(NodeChanged {
            after,
            before,
            cluster_id,
            node_id,
        });
        let payload = Payload::Node(event);
        self.builder.finish(payload)
    }

    /// Build a `NodeEvent::New` event.
    pub fn new_node(self, node: Node) -> Event {
        let event = NodeEvent::New(node);
        let payload = Payload::Node(event);
        self.builder.finish(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use super::NodeChanged;
    use super::NodeEvent;
    use super::Payload;
    use crate::agent::Node;

    #[test]
    fn changed() {
        let after = Node {
            cluster_display_name: None,
            cluster_id: "cluster".into(),
            kind: "TestDB".into(),
            node_id: "node".into(),
            version: "4.5.6".into(),
        };
        let before = Node {
            cluster_display_name: None,
            cluster_id: "cluster".into(),
            kind: "TestDB".into(),
            node_id: "node".into(),
            version: "1.2.3".into(),
        };
        let event = Event::builder()
            .node()
            .changed(before.clone(), after.clone());
        let expected = Payload::Node(NodeEvent::Changed(NodeChanged {
            after,
            cluster_id: before.cluster_id.clone(),
            node_id: before.node_id.clone(),
            before,
        }));
        assert_eq!(event.payload, expected);
    }

    #[test]
    fn new_node() {
        let node = Node {
            cluster_display_name: None,
            cluster_id: "cluster".into(),
            kind: "TestDB".into(),
            node_id: "node".into(),
            version: "1.2.3".into(),
        };
        let event = Event::builder().node().new_node(node.clone());
        let expected = Payload::Node(NodeEvent::New(node));
        assert_eq!(event.payload, expected);
    }
}
