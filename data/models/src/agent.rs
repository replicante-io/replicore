use replicante_agent_models::AgentInfo as WireAgentInfo;


/// Status of an agent.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub cluster: String,
    pub host: String,
    pub status: AgentStatus,
}

impl Agent {
    pub fn new<S1, S2>(cluster: S1, host: S2, status: AgentStatus) -> Agent
        where S1: Into<String>,
              S2: Into<String>,
    {
        Agent {
            cluster: cluster.into(),
            host: host.into(),
            status: status,
        }
    }
}


/// Information about an Agent
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub cluster: String,
    pub host: String,
    pub version_checkout: String,
    pub version_number: String,
    pub version_taint: String,
}

impl AgentInfo {
    pub fn new<S1, S2>(cluster: S1, host: S2, agent: WireAgentInfo) -> AgentInfo
        where S1: Into<String>,
              S2: Into<String>,
    {
        AgentInfo {
            cluster: cluster.into(),
            host: host.into(),
            version_checkout: agent.version.checkout,
            version_number: agent.version.number,
            version_taint: agent.version.taint,
        }
    }
}


/// Tracks the last known state of an agent.
///
/// If an agent or its datastore are down the received error is attached.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum AgentStatus {
    /// The agent is down or is returning errors.
    AgentDown(String),

    /// The agent is up but the datastore is down or is returning errors.
    DatastoreDown(String),

    /// The agent is up and can communicate with the datastore.
    Up,
}


#[cfg(test)]
mod tests {
    mod agent {
        use serde_json;
        use super::super::Agent;
        use super::super::AgentStatus;

        #[test]
        fn from_json() {
            let status = AgentStatus::AgentDown("TEST".into());
            let expected = Agent::new("cluster", "http://node/", status);
            let payload = r#"{"cluster":"cluster","host":"http://node/","status":{"AgentDown":"TEST"}}"#;
            let agent: Agent = serde_json::from_str(payload).unwrap();
            assert_eq!(agent, expected);
        }

        #[test]
        fn to_json() {
            let status = AgentStatus::AgentDown("TEST".into());
            let agent = Agent::new("cluster", "http://node/", status);
            let payload = serde_json::to_string(&agent).unwrap();
            let expected = r#"{"cluster":"cluster","host":"http://node/","status":{"AgentDown":"TEST"}}"#;
            assert_eq!(payload, expected);
        }
    }

    mod agent_info {
        use serde_json;
        use replicante_agent_models::AgentInfo as WireAgentInfo;
        use replicante_agent_models::AgentVersion as WireAgentVersion;
        use super::super::AgentInfo;

        #[test]
        fn from_json() {
            let version = WireAgentVersion::new("check", "1.2.3", "yep");
            let wire = WireAgentInfo::new(version);
            let expected = AgentInfo::new("cluster", "http://node/", wire);
            let payload = r#"{"cluster":"cluster","host":"http://node/","version_checkout":"check","version_number":"1.2.3","version_taint":"yep"}"#;
            let info: AgentInfo = serde_json::from_str(&payload).unwrap();
            assert_eq!(info, expected);
        }

        #[test]
        fn to_json() {
            let version = WireAgentVersion::new("check", "1.2.3", "yep");
            let wire = WireAgentInfo::new(version);
            let info = AgentInfo::new("cluster", "http://node/", wire);
            let payload = serde_json::to_string(&info).unwrap();
            let expected = r#"{"cluster":"cluster","host":"http://node/","version_checkout":"check","version_number":"1.2.3","version_taint":"yep"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod agent_status {
        use serde_json;
        use super::super::AgentStatus;

        #[test]
        fn agent_down() {
            let status = AgentStatus::AgentDown("TEST".into());
            let payload = serde_json::to_string(&status).unwrap();
            let expected = r#"{"AgentDown":"TEST"}"#;
            assert_eq!(payload, expected);
        }

        #[test]
        fn downstore_down() {
            let status = AgentStatus::DatastoreDown("TEST".into());
            let payload = serde_json::to_string(&status).unwrap();
            let expected = r#"{"DatastoreDown":"TEST"}"#;
            assert_eq!(payload, expected);
        }

        #[test]
        fn up() {
            let status = AgentStatus::Up;
            let payload = serde_json::to_string(&status).unwrap();
            let expected = r#""Up""#;
            assert_eq!(payload, expected);
        }
    }
}
