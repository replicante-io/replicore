use std::collections::HashMap;
use std::sync::Mutex;

use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_models_agent::actions::ActionListItem;
use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;

use crate::Client;
use crate::ErrorKind;
use crate::Result;

/// A mock `Client` for tests.
pub struct MockClient<A, D, S>
where
    A: Fn() -> Result<AgentInfo>,
    D: Fn() -> Result<DatastoreInfo>,
    S: Fn() -> Result<Shards>,
{
    agent_info: A,
    datastore_info: D,
    shards: S,
    pub actions: HashMap<Uuid, ActionInfoResponse>,
    pub actions_finished: Vec<ActionListItem>,
    pub actions_queue: Vec<ActionListItem>,
    pub actions_to_schedule: Mutex<Vec<(String, ActionScheduleRequest)>>,
    pub id: String,
}

impl<A, D, S> Client for MockClient<A, D, S>
where
    A: Fn() -> Result<AgentInfo>,
    D: Fn() -> Result<DatastoreInfo>,
    S: Fn() -> Result<Shards>,
{
    fn action_info(&self, id: &Uuid, _: Option<SpanContext>) -> Result<ActionInfoResponse> {
        let action = self
            .actions
            .get(id)
            .cloned()
            .ok_or_else(|| ErrorKind::NotFound("action", id.to_string()))?;
        Ok(action)
    }

    fn actions_finished(&self, _: Option<SpanContext>) -> Result<Vec<ActionListItem>> {
        Ok(self.actions_finished.clone())
    }

    fn actions_queue(&self, _: Option<SpanContext>) -> Result<Vec<ActionListItem>> {
        Ok(self.actions_queue.clone())
    }

    fn agent_info(&self, _: Option<SpanContext>) -> Result<AgentInfo> {
        (self.agent_info)()
    }

    fn datastore_info(&self, _: Option<SpanContext>) -> Result<DatastoreInfo> {
        (self.datastore_info)()
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn shards(&self, _: Option<SpanContext>) -> Result<Shards> {
        (self.shards)()
    }

    fn schedule_action(
        &self,
        kind: &str,
        _: &HashMap<String, String>,
        request: ActionScheduleRequest,
        _: Option<SpanContext>,
    ) -> Result<()> {
        self.actions_to_schedule
            .lock()
            .expect("agent MockClient::actions_to_schedule lock poisoned")
            .push((kind.to_string(), request));
        Ok(())
    }
}

impl<A, D, S> MockClient<A, D, S>
where
    A: Fn() -> Result<AgentInfo>,
    D: Fn() -> Result<DatastoreInfo>,
    S: Fn() -> Result<Shards>,
{
    /// Creates a new `MockClient`.
    pub fn new(agent_info: A, datastore_info: D, shards: S) -> MockClient<A, D, S> {
        let id = "mock://agent".to_string();
        MockClient {
            actions: HashMap::new(),
            actions_finished: Vec::new(),
            actions_queue: Vec::new(),
            actions_to_schedule: Mutex::new(Vec::new()),
            agent_info,
            datastore_info,
            id,
            shards,
        }
    }
}

#[cfg(test)]
mod tests {
    use replicante_models_agent::info::AgentInfo;
    use replicante_models_agent::info::AgentVersion;
    use replicante_models_agent::info::CommitOffset;
    use replicante_models_agent::info::DatastoreInfo;
    use replicante_models_agent::info::Shard;
    use replicante_models_agent::info::ShardRole;
    use replicante_models_agent::info::Shards;

    fn mock_agent_info() -> AgentInfo {
        AgentInfo::new(AgentVersion::new("a", "b", "c"))
    }

    fn mock_datastore_info() -> DatastoreInfo {
        DatastoreInfo::new("a", "b", "c", "d", None)
    }

    fn mock_shards() -> Shards {
        let shard = Shard::new(
            "id",
            ShardRole::Primary,
            Some(CommitOffset::seconds(1234)),
            Some(CommitOffset::seconds(2)),
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
            match client.agent_info(None) {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result"),
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
            assert_eq!(info, client.agent_info(None).unwrap());
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
            match client.datastore_info(None) {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result"),
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
            assert_eq!(info, client.datastore_info(None).unwrap());
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
            match client.shards(None) {
                Err(error) => match error.kind() {
                    &ErrorKind::Remote(ref msg) => assert_eq!("TestError", msg),
                    _ => panic!("Unexpected Err result: {:?}", error),
                },
                Ok(_) => panic!("Unexpected Ok result"),
            };
        }

        #[test]
        fn ok() {
            let info = mock_shards();
            let client = MockClient::new(
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Err(ErrorKind::Remote("Skipped".into()).into()),
                || Ok(mock_shards()),
            );
            assert_eq!(info, client.shards(None).unwrap());
        }
    }
}
