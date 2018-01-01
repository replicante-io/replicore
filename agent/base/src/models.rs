/// Stores agent version details.
#[derive(Clone, Debug, Serialize)]
pub struct AgentVersion {
    checkout: String,
    number: String,
    taint: String,
}

impl AgentVersion {
    pub fn new(checkout: &str, number: &str, taint: &str) -> AgentVersion {
        AgentVersion {
            checkout: String::from(checkout),
            number: String::from(number),
            taint: String::from(taint)
        }
    }
}


/// Stores datastore version details.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct DatastoreVersion {
    name: String,
    version: String,
}

impl DatastoreVersion {
    pub fn new(name: &str, version: &str) -> DatastoreVersion {
        DatastoreVersion {
            name: String::from(name),
            version: String::from(version)
        }
    }
}


/// Stores individual shard information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct Shard {
    id: String,
    role: ShardRole,
    lag: i64,
    last_op: i64,
}

impl Shard {
    pub fn new(id: &str, role: ShardRole, lag: i64, last_op: i64) -> Shard {
        Shard {
            id: String::from(id),
            role, lag, last_op
        }
    }
}


/// Enumeration of possible shard roles.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub enum ShardRole {
    Primary,
    Secondary,
    Unknown(String)
}


#[cfg(test)]
mod tests {
    mod agent_version {
        use serde_json;
        use super::super::AgentVersion;

        #[test]
        fn to_json() {
            let version = AgentVersion::new("abc123", "1.2.3", "tainted");
            let payload = serde_json::to_string(&version).unwrap();
            let expected = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod datastore_version {
        use serde_json;
        use super::super::DatastoreVersion;

        #[test]
        fn from_json() {
            let payload = r#"{"name":"DB","version":"1.2.3"}"#;
            let version: DatastoreVersion = serde_json::from_str(payload).unwrap();
            let expected = DatastoreVersion::new("DB", "1.2.3");
            assert_eq!(version, expected);
        }

        #[test]
        fn to_json() {
            let version = DatastoreVersion::new("DB", "1.2.3");
            let payload = serde_json::to_string(&version).unwrap();
            let expected = r#"{"name":"DB","version":"1.2.3"}"#;
            assert_eq!(payload, expected);
        }
    }

    mod shard {
        use serde_json;
        use super::super::Shard;
        use super::super::ShardRole;

        #[test]
        fn primary_from_json() {
            let payload = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new("shard-1", ShardRole::Primary, 0, 12345);
            assert_eq!(shard, expected);
        }

        #[test]
        fn primary_to_json() {
            let shard = Shard::new("shard-1", ShardRole::Primary, 0, 12345);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
            assert_eq!(payload, expected);
        }

        #[test]
        fn unkown_from_json() {
            let payload = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let expected = Shard::new(
                "shard-1", ShardRole::Unknown(String::from("Test")),
                0, 12345
            );
            assert_eq!(shard, expected);
        }

        #[test]
        fn unkown_to_json() {
            let shard = Shard::new(
                "shard-1", ShardRole::Unknown(String::from("Test")),
                0, 12345
            );
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
            assert_eq!(payload, expected);
        }
    }
}
