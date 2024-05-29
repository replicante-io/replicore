//! Platform API client for the HTTP(S) protocol.
use anyhow::Result;
use reqwest::Client as ReqwestClient;

use replisdk::platform::models::ClusterDiscoveryResponse;
use replisdk::platform::models::NodeDeprovisionRequest;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;

use repliclient_utils::ClientOptions;
use repliplatform_client::IPlatform;

/// String to set as the user agent in HTTP request.
static CLIENT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Platform API client for the HTTP(S) protocol.
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
impl IPlatform for HttpClient {
    async fn deprovision(&self, request: NodeDeprovisionRequest) -> Result<()> {
        let response = self
            .client
            .post(format!("{}deprovision", self.base))
            .json(&request)
            .send()
            .await?;
        repliclient_utils::inspect::<()>(response).await?;
        Ok(())
    }

    async fn discover(&self) -> Result<ClusterDiscoveryResponse> {
        let response = self
            .client
            .get(format!("{}discover", self.base))
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }

    async fn provision(&self, request: NodeProvisionRequest) -> Result<NodeProvisionResponse> {
        let response = self
            .client
            .post(format!("{}provision", self.base))
            .json(&request)
            .send()
            .await?;
        match repliclient_utils::inspect(response).await? {
            None => anyhow::bail!(repliclient_utils::EmptyResponse),
            Some(response) => Ok(response),
        }
    }
}
