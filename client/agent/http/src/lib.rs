//! Agent API client for the HTTP(S) protocol.
use anyhow::Result;
use reqwest::Client as ReqwestClient;
use uuid::Uuid;

use replisdk::agent::models::ActionExecution;
use replisdk::agent::models::ActionExecutionList;
use replisdk::agent::models::ActionExecutionRequest;
use replisdk::agent::models::ActionExecutionResponse;
use replisdk::agent::models::Node;
use replisdk::agent::models::ShardsInfo;
use replisdk::agent::models::StoreExtras;

use repliagent_client::IAgent;

pub use repliclient_utils::ClientOptions;

/// String to set as the user agent in HTTP request.
static CLIENT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Agent API client for the HTTP(S) protocol.
#[derive(Default)]
pub struct HttpClient {
    /// Base URL of the API server to send requests to.
    base: String,

    /// Low-level [`Client`](reqwest::Client) to perform HTTP requests with.
    client: ReqwestClient,
}

impl HttpClient {
    /// Initialise a client with [`ClientOptions`].
    pub fn with<O>(options: O) -> Result<HttpClient>
    where
        O: Into<ClientOptions>,
    {
        let options = options.into();
        let client = options.client(CLIENT_USER_AGENT);
        // TODO: TLS options
        let client = HttpClient {
            base: options.address,
            client: client.build()?,
        };
        Ok(client)
    }
}

#[async_trait::async_trait]
impl IAgent for HttpClient {
    /// Lookup a node action from the agent using an action ID.
    async fn action_lookup(&self, action: Uuid) -> Result<ActionExecution> {
        let response = self
            .client
            .get(format!("{}unstable/action/{}", self.base, action))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Send a request for the agent to schedule a node action. 
    async fn action_schedule(
        &self,
        action: ActionExecutionRequest,
    ) -> Result<ActionExecutionResponse> {
        let response = self
            .client
            .post(format!("{}unstable/action", self.base))
            .json(&action)
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Query an agent for a list of finished node actions.
    async fn actions_finished(&self) -> Result<ActionExecutionList> {
        let response = self
            .client
            .get(format!("{}unstable/actions/finished", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Query an agent for a list of running and queued node actions.
    async fn actions_queue(&self) -> Result<ActionExecutionList> {
        let response = self
            .client
            .get(format!("{}unstable/actions/queue", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Query an agent for information about the node, even when the store is not running.
    async fn info_node(&self) -> Result<Node> {
        let response = self
            .client
            .get(format!("{}unstable/info/node", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Query and agent for information about all shards managed by the node.
    async fn info_shards(&self) -> Result<ShardsInfo> {
        let response = self
            .client
            .get(format!("{}unstable/info/shards", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    /// Query an agent for information only available when the store process is healthy.
    async fn info_store(&self) -> Result<StoreExtras> {
        let response = self
            .client
            .get(format!("{}unstable/info/store", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }
}
