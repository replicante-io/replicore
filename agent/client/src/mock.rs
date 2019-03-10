use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::Client;
use super::Result;


/// A mock `Client` for tests.
pub struct MockClient<A, D, S>
    where A: Fn() -> Result<AgentInfo>,
          D: Fn() -> Result<DatastoreInfo>,
          S: Fn() -> Result<Shards>,
{
    agent_info: A,
    datastore_info: D,
    shards: S,
    pub id: String,
}

impl<A, D, S> Client for MockClient<A, D, S>
    where A: Fn() -> Result<AgentInfo>,
          D: Fn() -> Result<DatastoreInfo>,
          S: Fn() -> Result<Shards>,
{
    fn agent_info(&self) -> Result<AgentInfo> {
        (self.agent_info)()
    }

    fn datastore_info(&self) -> Result<DatastoreInfo> {
        (self.datastore_info)()
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn shards(&self) -> Result<Shards> {
        (self.shards)()
    }
}

impl<A, D, S> MockClient<A, D, S>
    where A: Fn() -> Result<AgentInfo>,
          D: Fn() -> Result<DatastoreInfo>,
          S: Fn() -> Result<Shards>,
{
    /// Creates a new `MockClient`.
    pub fn new(agent_info: A, datastore_info: D, shards: S) -> MockClient<A, D, S> {
        let id = "mock://agent".to_string();
        MockClient { agent_info, datastore_info, id, shards }
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::AgentVersion;
    use replicante_agent_models::DatastoreInfo;
    use replicante_agent_models::CommitOffset;
    use replicante_agent_models::Shard;
    use replicante_agent_models::Shards;
    use replicante_agent_models::ShardRole;

    fn mock_agent_info() -> AgentInfo {
        AgentInfo::new(AgentVersion::new("a", "b", "c"))
    }

    fn mock_datastore_info() -> DatastoreInfo {
        DatastoreInfo::new("a", "b", "c", "d")
    }

    fn mock_shards() -> Shards {
        let shard = Shard::new(
            "id", ShardRole::Primary, Some(CommitOffset::seconds(1234)),
            Some(CommitOffset::seconds(2))
        );
        Shards::new(vec![shard])
    }

    mod agent {
        use super::super::super::ErrorKind;
        use super::super::Client;
        use super::super::MockClient;
        use super::mock_agent_info;

        #[test]
        fn err() {
            let client = MockClient::new(
                || Err(ErrorKind::Remote("TestError".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
            );
            match client.agent_info() {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result")
            };
        }

        #[test]
        fn ok() {
            let info = mock_agent_info();
            let client = MockClient::new(
                || Ok(mock_agent_info()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
            );
            assert_eq!(info, client.agent_info().unwrap());
        }
    }

    mod datastore {
        use super::super::super::ErrorKind;
        use super::super::Client;
        use super::super::MockClient;
        use super::mock_datastore_info;

        #[test]
        fn err() {
            let client = MockClient::new(
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("TestError".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
            );
            match client.datastore_info() {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result")
            };
        }

        #[test]
        fn ok() {
            let info = mock_datastore_info();
            let client = MockClient::new(
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Ok(mock_datastore_info()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
            );
            assert_eq!(info, client.datastore_info().unwrap());
        }
    }

    mod shards {
        use super::super::super::ErrorKind;
        use super::super::Client;
        use super::super::MockClient;
        use super::mock_shards;

        #[test]
        fn err() {
            let client = MockClient::new(
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("TestError".into()).into()),
            );
            match client.shards() {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result")
            };
        }

        #[test]
        fn ok() {
            let info = mock_shards();
            let client = MockClient::new(
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Ok(mock_shards())
            );
            assert_eq!(info, client.shards().unwrap());
        }
    }
}
