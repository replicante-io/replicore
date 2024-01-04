//! Implement the list methods for API clients.
use anyhow::Result;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::ClusterSpecList;
use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::NamespaceList;
use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::api::PlatformList;

use super::Client;
use crate::error::EmptyResponse;

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
        let response = crate::error::inspect::<ClusterSpecList>(response).await?;
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
        let response = crate::error::inspect::<NamespaceList>(response).await?;
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
        let response = crate::error::inspect::<PlatformList>(response).await?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response.items)
    }
}
