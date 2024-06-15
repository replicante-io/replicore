//! Async client library to interact with Replicante Agent API.
use anyhow::Result;
use uuid::Uuid;

use replisdk::agent::models::ActionExecution;
use replisdk::agent::models::ActionExecutionList;
use replisdk::agent::models::ActionExecutionRequest;
use replisdk::agent::models::ActionExecutionResponse;
use replisdk::agent::models::Node;
use replisdk::agent::models::ShardsInfo;
use replisdk::agent::models::StoreExtras;

mod error;

#[cfg(any(test, feature = "test-fixture"))]
pub mod fixture;

pub use self::error::ActionNotFound;

/// Async API client to Replicante Agents.
pub struct Client {
    backend: Box<dyn IAgent>,
}

impl Client {
    /// Lookup a node action from the agent using an action ID.
    pub async fn action_lookup(&self, action: Uuid) -> Result<ActionExecution> {
        self.backend.action_lookup(action).await
    }

    /// Send a request for the agent to schedule a node action.
    pub async fn action_schedule(
        &self,
        action: ActionExecutionRequest,
    ) -> Result<ActionExecutionResponse> {
        self.backend.action_schedule(action).await
    }

    /// Query an agent for a list of finished node actions.
    pub async fn actions_finished(&self) -> Result<ActionExecutionList> {
        self.backend.actions_finished().await
    }

    /// Query an agent for a list of running and queued node actions.
    pub async fn actions_queue(&self) -> Result<ActionExecutionList> {
        self.backend.actions_queue().await
    }

    /// Query an agent for information about the node, even when the store is not running.
    pub async fn info_node(&self) -> Result<Node> {
        self.backend.info_node().await
    }

    /// Query and agent for information about all shards managed by the node.
    pub async fn info_shards(&self) -> Result<ShardsInfo> {
        self.backend.info_shards().await
    }

    /// Query an agent for information only available when the store process is healthy.
    pub async fn info_store(&self) -> Result<StoreExtras> {
        self.backend.info_store().await
    }
}

impl<P> From<P> for Client
where
    P: IAgent + 'static,
{
    fn from(value: P) -> Self {
        let backend = Box::new(value);
        Client { backend }
    }
}

/// Interface to Agent API clients.
///
/// Enables implementation of Agent API clients across different transport protocols.
#[async_trait::async_trait]
pub trait IAgent: Send + Sync {
    /// Lookup a node action from the agent using an action ID.
    async fn action_lookup(&self, action: Uuid) -> Result<ActionExecution>;

    /// Send a request for the agent to schedule a node action.
    async fn action_schedule(
        &self,
        action: ActionExecutionRequest,
    ) -> Result<ActionExecutionResponse>;

    /// Query an agent for a list of finished node actions.
    async fn actions_finished(&self) -> Result<ActionExecutionList>;

    /// Query an agent for a list of running and queued node actions.
    async fn actions_queue(&self) -> Result<ActionExecutionList>;

    /// Query an agent for information about the node, even when the store is not running.
    async fn info_node(&self) -> Result<Node>;

    /// Query and agent for information about all shards managed by the node.
    async fn info_shards(&self) -> Result<ShardsInfo>;

    /// Query an agent for information only available when the store process is healthy.
    async fn info_store(&self) -> Result<StoreExtras>;
}
