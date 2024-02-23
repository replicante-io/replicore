//! Async client library to interact with Replicante Platform integrations API.
use anyhow::Result;

use replisdk::platform::models::ClusterDiscoveryResponse;
use replisdk::platform::models::NodeDeprovisionRequest;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;

#[cfg(any(test, feature = "test-fixture"))]
pub mod fixture;

/// Async API client to Replicante Platform integrations.
pub struct Client {
    backend: Box<dyn IPlatform>,
}

impl Client {
    /// Request the deprovisioning of a node.
    pub async fn deprovision(&self, request: NodeDeprovisionRequest) -> Result<()> {
        self.backend.deprovision(request).await
    }

    /// Returns all clusters on the Platform.
    pub async fn discover(&self) -> Result<ClusterDiscoveryResponse> {
        self.backend.discover().await
    }

    /// Request the provisioning of new node(s).
    pub async fn provision(&self, request: NodeProvisionRequest) -> Result<NodeProvisionResponse> {
        self.backend.provision(request).await
    }
}

impl<P> From<P> for Client
where
    P: IPlatform + 'static,
{
    fn from(value: P) -> Self {
        let backend = Box::new(value);
        Client { backend }
    }
}

/// Interface to Platform API clients.
///
/// Enables implementation of Platform API clients across different transport protocols.
#[async_trait::async_trait]
pub trait IPlatform: Send + Sync {
    /// Request the deprovisioning of a node.
    async fn deprovision(&self, request: NodeDeprovisionRequest) -> Result<()>;

    /// Returns all clusters on the Platform.
    async fn discover(&self) -> Result<ClusterDiscoveryResponse>;

    /// Request the provisioning of new node(s).
    async fn provision(&self, request: NodeProvisionRequest) -> Result<NodeProvisionResponse>;
}
