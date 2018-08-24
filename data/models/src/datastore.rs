use replicante_agent_models::DatastoreInfo as WireNode;
use replicante_agent_models::Shard as WireShard;

// Re-export some models for core to use.
// This opens up the option of replacing the implementation without changing dependants.
pub use replicante_agent_models::CommitOffset;
pub use replicante_agent_models::CommitUnit;
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
    pub commit_offset: Option<CommitOffset>,
    pub id: String,
    pub lag: Option<CommitOffset>,
    pub node: String,
    pub role: ShardRole,
}

impl Shard {
    pub fn new<S1, S2>(cluster: S1, node: S2, shard: WireShard) -> Shard
        where S1: Into<String>,
              S2: Into<String>,
    {
        Shard {
            cluster: cluster.into(),
            commit_offset: shard.commit_offset,
            id: shard.id,
            lag: shard.lag,
            node: node.into(),
            role: shard.role,
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
        use replicante_agent_models::CommitOffset;
        use replicante_agent_models::Shard as WireShard;
        use replicante_agent_models::ShardRole;
        use super::super::Shard;

        #[test]
        fn from_json() {
            let payload = concat!(
                r#"{"cluster":"cluster","commit_offset":{"unit":"seconds","value":54},"#,
                r#""id":"shard","lag":null,"node":"node","role":"secondary"}"#
            );
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let wire = WireShard::new(
                "shard", ShardRole::Secondary,
                Some(CommitOffset::seconds(54)), None
            );
            let expected = Shard::new("cluster", "node", wire);
            assert_eq!(shard, expected);
        }

        #[test]
        fn to_json() {
            let wire = WireShard::new(
                "shard", ShardRole::Secondary,
                Some(CommitOffset::seconds(54)), None
            );
            let shard = Shard::new("cluster", "node", wire);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = concat!(
                r#"{"cluster":"cluster","commit_offset":{"unit":"seconds","value":54},"#,
                r#""id":"shard","lag":null,"node":"node","role":"secondary"}"#
            );
            assert_eq!(payload, expected);
        }
    }
}
