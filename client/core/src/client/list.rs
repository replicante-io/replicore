//! Implement the list methods for API clients.
use anyhow::Result;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::ClusterSpecList;
use replisdk::core::models::api::NActionEntry;
use replisdk::core::models::api::NActionList;
use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::NamespaceList;
use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::api::OActionList;
use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::api::PlatformList;

use repliclient_utils::EmptyResponse;

use super::Client;

/// Access resource listing operations.
pub struct ListClient<'a> {
    inner: &'a Client,
}

impl Client {
    /// Resource listing operations.
    pub fn list(&self) -> ListClient {
        ListClient { inner: self }
    }
}

impl<'a> ListClient<'a> {
    /// List cluster specifications known to the control plane, scoped to a namespace.
    pub async fn clusterspecs(&'a self, namespace: &str) -> Result<Vec<ClusterSpecEntry>> {
        let response = self
            .inner
            .client
            .get(format!(
                "{}api/v0/list/replicante.io/v0/clusterspec/{}",
                self.inner.base, namespace,
            ))
            .send()
            .await?;
        let response = repliclient_utils::inspect::<ClusterSpecList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }

    /// List node actions for a cluster.
    pub async fn nactions(
        &'a self,
        namespace: &str,
        cluster: &str,
        node: &Option<String>,
        all: bool,
    ) -> Result<Vec<NActionEntry>> {
        let request = self
            .inner
            .client
            .get(format!(
                "{}api/v0/list/replicante.io/v0/naction/{}/{}",
                self.inner.base, namespace, cluster,
            ))
            .query(&[("all", all)]);
        let request = match node {
            None => request,
            Some(node) => request.query(&[("node_id", node)]),
        };
        let response = request.send().await?;
        let response = repliclient_utils::inspect::<NActionList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }

    /// List namespaces known to the control plane.
    pub async fn namespaces(&'a self) -> Result<Vec<NamespaceEntry>> {
        let response = self
            .inner
            .client
            .get(format!(
                "{}api/v0/list/replicante.io/v0/namespace",
                self.inner.base,
            ))
            .send()
            .await?;
        let response = repliclient_utils::inspect::<NamespaceList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }

    /// List orchestrator actions for a cluster.
    pub async fn oactions(
        &'a self,
        namespace: &str,
        cluster: &str,
        all: bool,
    ) -> Result<Vec<OActionEntry>> {
        let response = self
            .inner
            .client
            .get(format!(
                "{}api/v0/list/replicante.io/v0/oaction/{}/{}",
                self.inner.base, namespace, cluster,
            ))
            .query(&[("all", all)])
            .send()
            .await?;
        let response = repliclient_utils::inspect::<OActionList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }

    /// List platform known to the control plane, scoped to a namespace.
    pub async fn platforms(&'a self, namespace: &str) -> Result<Vec<PlatformEntry>> {
        let response = self
            .inner
            .client
            .get(format!(
                "{}api/v0/list/replicante.io/v0/platform/{}",
                self.inner.base, namespace,
            ))
            .send()
            .await?;
        let response = repliclient_utils::inspect::<PlatformList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }
}
