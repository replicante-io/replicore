use replicante_agent_models::DatastoreInfo as WireNode;
use replicante_agent_models::Shard as WireShard;

// Re-export the ShardRole for core to use.
// This opens up the option of replacing the implementation without changing dependants.
pub use replicante_agent_models::ShardRole;


/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Node {
    pub cluster: String,
    pub kind: String,
    pub name: String,
    pub version: String,
}

impl Node {
    pub fn new(node: WireNode) -> Node {
        Node {
            cluster: node.cluster,
            kind: node.kind,
            name: node.name,
            version: node.version,
        }
    }
}


/// Information about a shard on a node.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Shard {
    pub cluster: String,
    pub node: String,
    pub id: String,
    pub role: ShardRole,
    pub lag: Option<i64>,
    pub last_op: i64,
}

impl Shard {
    pub fn new<S1, S2>(cluster: S1, node: S2, shard: WireShard) -> Shard
        where S1: Into<String>,
              S2: Into<String>,
    {
        Shard {
            cluster: cluster.into(),
            node: node.into(),
            id: shard.id,
            role: shard.role,
            lag: shard.lag,
            last_op: shard.last_op,
        }
    }
}


#[cfg(test)]
mod tests {
    mod node {
        use serde_json;
        use replicante_agent_models::DatastoreInfo as WireNode;
        use super::super::Node;

        #[test]
        fn from_json() {
            let payload = r#"{"cluster":"cluster","kind":"DB","name":"Name","version":"1.2.3"}"#;
            let node: Node = serde_json::from_str(payload).unwrap();
            let wire = WireNode::new("cluster", "DB", "Name", "1.2.3");
            let expected = Node::new(wire);
            assert_eq!(node, expected);
        }

        #[test]
        fn to_json() {
            let wire = WireNode::new("cluster", "DB", "Name", "1.2.3");
            let node = Node::new(wire);
            let payload = serde_json::to_string(&node).unwrap();
            let expected = r#"{"cluster":"cluster","kind":"DB","name":"Name","version":"1.2.3"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod shard {
        use serde_json;
        use replicante_agent_models::Shard as WireShard;
        use replicante_agent_models::ShardRole;
        use super::super::Shard;

        #[test]
        fn from_json() {
            let payload = r#"{"cluster":"cluster","node":"node","id":"shard","role":"Secondary","lag":null,"last_op":54}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let wire = WireShard::new("shard", ShardRole::Secondary, None, 54);
            let expected = Shard::new("cluster", "node", wire);
            assert_eq!(shard, expected);
        }

        #[test]
        fn to_json() {
            let wire = WireShard::new("shard", ShardRole::Secondary, None, 54);
            let shard = Shard::new("cluster", "node", wire);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"cluster":"cluster","node":"node","id":"shard","role":"Secondary","lag":null,"last_op":54}"#;
            assert_eq!(payload, expected);
        }
    }
}
