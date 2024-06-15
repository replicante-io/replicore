//! Mock agent client implementation for unit tests.
//use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use uuid::Uuid;

use replisdk::agent::models::ActionExecution;
use replisdk::agent::models::ActionExecutionList;
use replisdk::agent::models::ActionExecutionListItem;
use replisdk::agent::models::ActionExecutionPhase;
use replisdk::agent::models::ActionExecutionRequest;
use replisdk::agent::models::ActionExecutionResponse;
use replisdk::agent::models::Node;
use replisdk::agent::models::Shard;
use replisdk::agent::models::ShardsInfo;
use replisdk::agent::models::StoreExtras;

/// Mock agent client implementation for unit tests.
pub struct Client {
    state: Mutex<ClientState>,
}

impl Client {
    /// Initialise a new client to mock a node.
    pub fn new(node: Node, store: StoreExtras) -> Client {
        let state = ClientState {
            finished: Default::default(),
            node,
            queue: Default::default(),
            shards: ShardsInfo {
                shards: Default::default(),
            },
            store,
        };
        Client {
            state: Mutex::new(state),
        }
    }

    /// Add an action execution to the node.
    pub fn action(&self, action: ActionExecution) -> &Self {
        let done = matches!(
            action.state.phase,
            ActionExecutionPhase::Done | ActionExecutionPhase::Failed
        );
        let mut state = self.state.lock().unwrap();
        if done {
            state.finished.push(action);
        } else {
            state.queue.push(action);
        };
        self
    }

    /// Add a shard to the node.
    pub fn shard(&self, shard: Shard) -> &Self {
        let mut state = self.state.lock().unwrap();
        state.shards.shards.push(shard);
        self
    }
}

#[async_trait::async_trait]
impl super::IAgent for Client {
    async fn action_lookup(&self, action: Uuid) -> Result<ActionExecution> {
        let state = self.state.lock().unwrap();
        match state.queue.iter().find(|a| a.id == action) {
            None => (),
            Some(action) => return Ok(action.clone()),
        };
        match state.finished.iter().find(|a| a.id == action) {
            None => anyhow::bail!(crate::ActionNotFound { action_id: action }),
            Some(action) => Ok(action.clone()),
        }
    }

    async fn action_schedule(
        &self,
        action: ActionExecutionRequest,
    ) -> Result<ActionExecutionResponse> {
        let action: ActionExecution = action.into();
        let action_id = action.id;
        let mut state = self.state.lock().unwrap();
        state.queue.push(action);
        Ok(ActionExecutionResponse { id: action_id })
    }

    async fn actions_finished(&self) -> Result<ActionExecutionList> {
        let state = self.state.lock().unwrap();
        let actions = state
            .finished
            .iter()
            .map(|action| ActionExecutionListItem {
                id: action.id,
                kind: action.kind.clone(),
                phase: action.state.phase,
            })
            .collect();
        Ok(ActionExecutionList { actions })
    }

    async fn actions_queue(&self) -> Result<ActionExecutionList> {
        let state = self.state.lock().unwrap();
        let actions = state
            .queue
            .iter()
            .map(|action| ActionExecutionListItem {
                id: action.id,
                kind: action.kind.clone(),
                phase: action.state.phase,
            })
            .collect();
        Ok(ActionExecutionList { actions })
    }

    async fn info_node(&self) -> Result<Node> {
        let state = self.state.lock().unwrap();
        Ok(state.node.clone())
    }

    async fn info_shards(&self) -> Result<ShardsInfo> {
        let state = self.state.lock().unwrap();
        Ok(state.shards.clone())
    }

    async fn info_store(&self) -> Result<StoreExtras> {
        let state = self.state.lock().unwrap();
        Ok(state.store.clone())
    }
}

/// Internal state to implement agent mocking.
struct ClientState {
    finished: Vec<ActionExecution>,
    node: Node,
    queue: Vec<ActionExecution>,
    shards: ShardsInfo,
    store: StoreExtras,
}
