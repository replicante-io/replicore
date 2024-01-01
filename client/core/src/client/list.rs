//! Implement the list methods for API clients.
use anyhow::Result;

use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::NamespaceList;

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
    /// List namespaces known to the cluster.
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
}
