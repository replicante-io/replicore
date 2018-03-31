use replicante_agent_models::NodeInfo;
use replicante_agent_models::NodeStatus;


/// Snapshot view of the state of a node
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Node {
    info: NodeInfo,
    status: NodeStatus,
}

impl Node {
    pub fn new(info: NodeInfo, status: NodeStatus) -> Node {
        Node { info, status }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::AgentVersion;
    use replicante_agent_models::DatastoreInfo;
    use replicante_agent_models::NodeInfo;
    use replicante_agent_models::NodeStatus;
    use replicante_agent_models::Shard;
    use replicante_agent_models::ShardRole;

    use super::Node;

    #[test]
    fn from_json() {
        let payload = r#"{"info":{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"kind":"a","name":"b","version":"c"}},"status":{"shards":[{"id":"id","role":"Secondary","lag":4,"last_op":5}]}}"#;
        let node: Node = serde_json::from_str(payload).unwrap();
        let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
        let datastore = DatastoreInfo::new("a", "b", "c");
        let info = NodeInfo::new(agent, datastore);
        let status = NodeStatus::new(vec![Shard::new("id", ShardRole::Secondary, Some(4), 5)]);
        let expected = Node::new(info, status);
        assert_eq!(node, expected);
    }

    #[test]
    fn to_json() {
        let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
        let datastore = DatastoreInfo::new("a", "b", "c");
        let info = NodeInfo::new(agent, datastore);
        let status = NodeStatus::new(vec![Shard::new("id", ShardRole::Secondary, Some(4), 5)]);
        let node = Node::new(info, status);
        let payload = serde_json::to_string(&node).unwrap();
        let expected = r#"{"info":{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"kind":"a","name":"b","version":"c"}},"status":{"shards":[{"id":"id","role":"Secondary","lag":4,"last_op":5}]}}"#;
        assert_eq!(payload, expected);
    }
}
