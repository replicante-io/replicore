use replicante_agent_models::NodeInfo;
use replicante_agent_models::NodeStatus;

use super::Client;
use super::Result;


/// A mock `Client` for tests.
pub struct MockClient<Info, Status>
    where Info: Fn() -> Result<NodeInfo>,
          Status: Fn() -> Result<NodeStatus>,
{
    info_factory: Info,
    status_factory: Status,
}

impl<Info, Status> Client for MockClient<Info, Status>
    where Info: Fn() -> Result<NodeInfo>,
          Status: Fn() -> Result<NodeStatus>,
{
    fn info(&self) -> Result<NodeInfo> {
        (self.info_factory)()
    }

    fn status(&self) -> Result<NodeStatus> {
        (self.status_factory)()
    }
}

impl<Info, Status> MockClient<Info, Status>
    where Info: Fn() -> Result<NodeInfo>,
          Status: Fn() -> Result<NodeStatus>,
{
    /// Creates a new `MockClient`.
    pub fn new(info_factory: Info, status_factory: Status) -> MockClient<Info, Status> {
        MockClient { info_factory, status_factory }
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::AgentVersion;
    use replicante_agent_models::DatastoreInfo;
    use replicante_agent_models::NodeInfo;
    use replicante_agent_models::NodeStatus;
    use replicante_agent_models::Shard;
    use replicante_agent_models::ShardRole;

    use super::super::Error;
    use super::super::ErrorKind;
    use super::Client;
    use super::MockClient;

    fn mock_info() -> NodeInfo {
        let agent = AgentInfo::new(AgentVersion::new("a", "b", "c"));
        let datastore = DatastoreInfo::new("a", "b", "c");
        NodeInfo::new(agent, datastore)
    }

    fn mock_status() -> NodeStatus {
        let shard = Shard::new("id", ShardRole::Primary, Some(2), 1234);
        NodeStatus::new(vec![shard])
    }

    #[test]
    fn info_err() {
        let client = MockClient::new(|| Err("TestError".into()), || Err("Skipped".into()));
        match client.info() {
            Err(Error(ErrorKind::Msg(error), _)) => assert_eq!("TestError", error),
            Err(_) => panic!("Unexpected Err result"),
            Ok(_) => panic!("Unexpected Ok result")
        };
    }

    #[test]
    fn info_ok() {
        let info = mock_info();
        let client = MockClient::new(|| Ok(info.clone()), || Err("Skipped".into()));
        assert_eq!(info, client.info().unwrap());
    }

    #[test]
    fn status_err() {
        let client = MockClient::new(|| Err("Skipped".into()), || Err("TestError".into()));
        match client.status() {
            Err(Error(ErrorKind::Msg(error), _)) => assert_eq!("TestError", error),
            Err(_) => panic!("Unexpected Err result"),
            Ok(_) => panic!("Unexpected Ok result")
        };
    }

    #[test]
    fn status_ok() {
        let status = mock_status();
        let client = MockClient::new(|| Err("Skipped".into()), || Ok(status.clone()));
        assert_eq!(status, client.status().unwrap());
    }
}
