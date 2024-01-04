//! Implement the ClusterSpec methods for API clients.
use anyhow::Context;
use anyhow::Result;

use replisdk::core::models::cluster::ClusterSpec;

use super::Client;
use crate::error::EmptyResponse;
use crate::error::ResourceIdentifier;

/// Access ClusterSpec operations.
pub struct ClusterSpecClient<'a> {
    inner: &'a Client,
    name: &'a str,
    ns_id: &'a str,
}

impl Client {
    /// Namespace operations.
    pub fn clusterspec<'a>(&'a self, ns_id: &'a str, name: &'a str) -> ClusterSpecClient<'a> {
        ClusterSpecClient {
            inner: self,
            name,
            ns_id,
        }
    }
}

impl<'a> ClusterSpecClient<'a> {
    /// Delete a [`ClusterSpec`] object from the control plane.
    pub async fn delete(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.delete(url).send().await?;
        crate::error::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterspec", id)
            })?;
        Ok(())
    }

    /// Fetch a [`ClusterSpec`] record from the server.
    pub async fn get(&'a self) -> Result<ClusterSpec> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = crate::error::inspect::<ClusterSpec>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterspec", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }
}
