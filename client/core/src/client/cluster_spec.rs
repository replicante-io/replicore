//! Implement the ClusterSpec methods for API clients.
use anyhow::Context;
use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;

use repliclient_utils::EmptyResponse;
use repliclient_utils::ResourceIdentifier;
use replicore_cluster_models::OrchestrateReport;

use super::Client;

/// Access ClusterSpec operations.
pub struct ClusterSpecClient<'a> {
    inner: &'a Client,
    name: &'a str,
    ns_id: &'a str,
}

impl Client {
    /// ClusterSpec operations.
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
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterspec", id)
            })?;
        Ok(())
    }

    /// Fetch a [`ClusterDiscovery`] record from the server.
    pub async fn discovery(&'a self) -> Result<Option<ClusterDiscovery>> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}/discovery",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<ClusterDiscovery>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterdiscovery", id)
            })?;
        Ok(response)
    }

    /// Fetch a [`ClusterSpec`] record from the server.
    pub async fn get(&'a self) -> Result<ClusterSpec> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<ClusterSpec>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterspec", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }

    /// Schedule a background orchestration task for a [`ClusterSpec`].
    pub async fn orchestrate(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}/orchestrate",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("clusterspec", id)
            })?;
        Ok(())
    }

    /// Fetch an [`OrchestrateReport`] record from the server.
    pub async fn orchestrate_report(&'a self) -> Result<Option<OrchestrateReport>> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/clusterspec/{}/{}/orchestrate/report",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<OrchestrateReport>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("orchestratereport", id)
            })?;
        Ok(response)
    }
}
