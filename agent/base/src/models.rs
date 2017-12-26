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
#[derive(Clone, Debug, Serialize)]
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


#[cfg(test)]
mod tests {
    use serde_json;
    use super::AgentVersion;
    use super::DatastoreVersion;

    #[test]
    fn agent_version_serialises_to_json() {
        let version = AgentVersion::new("abc123", "1.2.3", "tainted");
        let payload = serde_json::to_string(&version).unwrap();
        let expected = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
        assert_eq!(payload, expected);
    }

    #[test]
    fn datastore_version_serialises_to_json() {
        let version = DatastoreVersion::new("DB", "1.2.3");
        let payload = serde_json::to_string(&version).unwrap();
        let expected = r#"{"name":"DB","version":"1.2.3"}"#;
        assert_eq!(payload, expected);
    }
}
