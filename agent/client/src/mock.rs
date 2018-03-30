use replicante_agent_models::AgentInfo;

use super::Client;
use super::Result;


/// A mock `Client` for tests.
pub struct MockClient<Info>
    where Info: Fn() -> Result<AgentInfo>,
{
    pub info_factory: Info,
}

impl<Info> Client for MockClient<Info>
    where Info: Fn() -> Result<AgentInfo>,
{
    fn info(&self) -> Result<AgentInfo> {
        (self.info_factory)()
    }
}

impl<Info> MockClient<Info>
    where Info: Fn() -> Result<AgentInfo>,
{
    /// Creates a new `MockClient`.
    pub fn new(info_factory: Info) -> MockClient<Info> {
        MockClient { info_factory }
    }
}


#[cfg(test)]
mod tests {
    use replicante_agent_models::AgentDetails;
    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::AgentVersion;
    use replicante_agent_models::DatastoreInfo;

    use super::super::Error;
    use super::super::ErrorKind;
    use super::Client;
    use super::MockClient;

    fn mock_info() -> AgentInfo {
        let agent = AgentDetails::new(AgentVersion::new("a", "b", "c"));
        let datastore = DatastoreInfo::new("a", "b", "c");
        AgentInfo::new(agent, datastore)
    }

    #[test]
    fn info_ok() {
        let info = mock_info();
        let client = MockClient::new(|| Ok(info.clone()));
        assert_eq!(info, client.info().unwrap());
    }

    #[test]
    fn info_err() {
        let client = MockClient::new(|| Err("TestError".into()));
        match client.info() {
            Err(Error(ErrorKind::Msg(error), _)) => assert_eq!("TestError", error),
            Err(_) => panic!("Unexpected Err result"),
            Ok(_) => panic!("Unexpected Ok result")
        };
    }
}
