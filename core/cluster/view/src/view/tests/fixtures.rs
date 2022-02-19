use replicante_models_agent::info::ShardRole;

use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

/// Fixtures for a fixtional MongoDB cluster.
pub mod cluster_mongodb {
    use super::*;

    pub const CLUSTER_ID: &str = "colours";
    pub const NAMESPACE: &str = "default";

    pub fn blue_node() -> Node {
        Node {
            cluster_display_name: None,
            cluster_id: CLUSTER_ID.into(),
            kind: "mongodb".into(),
            node_id: "https://blue.mongo.fixtures:12345/".into(),
            version: "4.5.6".into(),
        }
    }

    pub fn blue_node_agent() -> Agent {
        let status = AgentStatus::AgentDown("agent error".into());
        Agent::new(CLUSTER_ID, "https://blue.mongo.fixtures:12345/", status)
    }

    pub fn blue_node_agent_info() -> AgentInfo {
        AgentInfo {
            cluster_id: CLUSTER_ID.into(),
            host: "https://blue.mongo.fixtures:12345/".into(),
            version_checkout: "".into(),
            version_number: "1.2.3".into(),
            version_taint: "not tainted".into(),
        }
    }

    pub fn blue_node_shard_rgb() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://blue.mongo.fixtures:12345/".into(),
            role: ShardRole::Primary,
            shard_id: "rgb".into(),
        }
    }

    pub fn blue_node_shard_hex() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://blue.mongo.fixtures:12345/".into(),
            role: ShardRole::Secondary,
            shard_id: "hex".into(),
        }
    }

    pub fn discovery() -> ClusterDiscovery {
        ClusterDiscovery::new(CLUSTER_ID, vec![])
    }

    pub fn green_node() -> Node {
        Node {
            cluster_display_name: None,
            cluster_id: CLUSTER_ID.into(),
            kind: "mongodb".into(),
            node_id: "https://green.mongo.fixtures:12345/".into(),
            version: "6.5.4".into(),
        }
    }

    pub fn green_node_agent() -> Agent {
        let status = AgentStatus::Up;
        Agent::new(CLUSTER_ID, "https://green.mongo.fixtures:12345/", status)
    }

    pub fn green_node_agent_info() -> AgentInfo {
        AgentInfo {
            cluster_id: CLUSTER_ID.into(),
            host: "https://green.mongo.fixtures:12345/".into(),
            version_checkout: "".into(),
            version_number: "3.2.1".into(),
            version_taint: "tainted".into(),
        }
    }

    pub fn green_node_shard_rgb() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://green.mongo.fixtures:12345/".into(),
            role: ShardRole::Secondary,
            shard_id: "rgb".into(),
        }
    }

    pub fn green_node_shard_cmyk() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://green.mongo.fixtures:12345/".into(),
            role: ShardRole::Secondary,
            shard_id: "cmyk".into(),
        }
    }

    pub fn red_node() -> Node {
        Node {
            cluster_display_name: None,
            cluster_id: CLUSTER_ID.into(),
            kind: "mongodb".into(),
            node_id: "https://red.mongo.fixtures:12345/".into(),
            version: "4.5.6".into(),
        }
    }

    pub fn red_node_agent() -> Agent {
        let status = AgentStatus::NodeDown("node error".into());
        Agent::new(CLUSTER_ID, "https://red.mongo.fixtures:12345/", status)
    }

    pub fn red_node_agent_info() -> AgentInfo {
        AgentInfo {
            cluster_id: CLUSTER_ID.into(),
            host: "https://red.mongo.fixtures:12345/".into(),
            version_checkout: "".into(),
            version_number: "1.2.3".into(),
            version_taint: "tainted".into(),
        }
    }

    pub fn red_node_shard_hex() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://red.mongo.fixtures:12345/".into(),
            role: ShardRole::Primary,
            shard_id: "hex".into(),
        }
    }

    pub fn red_node_shard_cmyk() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://red.mongo.fixtures:12345/".into(),
            role: ShardRole::Primary,
            shard_id: "cmyk".into(),
        }
    }

    pub fn settings() -> ClusterSettings {
        ClusterSettings::synthetic(NAMESPACE, CLUSTER_ID)
    }
}

/// Fixtures for a fixtional Zookeeepr cluster.
pub mod cluster_zookeeper {
    use super::*;

    pub const CLUSTER_ID: &str = "animals";
    pub const NAMESPACE: &str = "default";

    pub fn dog_node() -> Node {
        Node {
            cluster_display_name: None,
            cluster_id: CLUSTER_ID.into(),
            kind: "zookeeper".into(),
            node_id: "https://dog.zookeeper.fixtures:12345/".into(),
            version: "4.5.6".into(),
        }
    }

    pub fn dog_node_agent() -> Agent {
        let status = AgentStatus::AgentDown("agent error".into());
        Agent::new(CLUSTER_ID, "https://dog.zookeeper.fixtures:12345/", status)
    }

    pub fn dog_node_agent_info() -> AgentInfo {
        AgentInfo {
            cluster_id: CLUSTER_ID.into(),
            host: "https://dog.zookeeper.fixtures:12345/".into(),
            version_checkout: "".into(),
            version_number: "1.2.3".into(),
            version_taint: "not tainted".into(),
        }
    }

    pub fn dog_node_shard_maltese() -> Shard {
        Shard {
            cluster_id: CLUSTER_ID.into(),
            commit_offset: None,
            lag: None,
            node_id: "https://dog.zookeeper.fixtures:12345/".into(),
            role: ShardRole::Primary,
            shard_id: "maltese".into(),
        }
    }

    pub fn settings() -> ClusterSettings {
        ClusterSettings::synthetic(NAMESPACE, CLUSTER_ID)
    }
}
