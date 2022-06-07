use std::str::FromStr;

use replicante_models_agent::info::ShardRole;

use replicante_models_core::actions::ActionState;
use replicante_models_core::actions::ActionSummary;
use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_models_core::agent::Node;
use replicante_models_core::agent::Shard;
use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

/// Fixtures for a fictional MongoDB cluster.
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

    pub fn blue_node_action_restart() -> ActionSummary {
        ActionSummary {
            cluster_id: CLUSTER_ID.into(),
            node_id: "https://blue.mongo.fixtures:12345/".into(),
            action_id: uuid::Uuid::from_str("0436430c-2b02-624c-2032-570501212b57").unwrap(),
            state: ActionState::New,
        }
    }

    pub fn blue_node_action_stepdown() -> ActionSummary {
        ActionSummary {
            action_id: uuid::Uuid::from_str("347db8f1-dab4-401b-8956-04cd0ca25661").unwrap(),
            ..blue_node_action_restart()
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

    pub fn green_node_action_stepdown() -> ActionSummary {
        ActionSummary {
            node_id: "https://green.mongo.fixtures:12345/".into(),
            action_id: uuid::Uuid::from_str("004089da-ec5a-4f4c-a4cc-adff9ec09015").unwrap(),
            ..blue_node_action_restart()
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

/// Fixtures for a fictional Zookeeepr cluster.
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
