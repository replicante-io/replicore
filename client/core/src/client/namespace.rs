//! Implement the namespace methods for API clients.
use anyhow::Context;
use anyhow::Result;

use replisdk::core::models::namespace::Namespace;

use repliclient_utils::EmptyResponse;
use repliclient_utils::ResourceIdentifier;

use super::Client;

/// Access namespace operations.
pub struct NamespaceClient<'a> {
    id: &'a str,
    inner: &'a Client,
}

impl Client {
    /// Namespace operations.
    pub fn namespace<'a>(&'a self, id: &'a str) -> NamespaceClient<'a> {
        NamespaceClient { inner: self, id }
    }
}

impl<'a> NamespaceClient<'a> {
    /// Set a [`Namespace`] to be deleted asynchronously.
    pub async fn delete(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/namespace/{}",
            self.inner.base, self.id,
        );
        let response = self.inner.client.delete(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| ResourceIdentifier::reference("namespace", self.id))?;
        Ok(())
    }

    /// Fetch a [`Namespace`] record from the server.
    pub async fn get(&'a self) -> Result<Namespace> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/namespace/{}",
            self.inner.base, self.id,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<Namespace>(response)
            .await
            .with_context(|| ResourceIdentifier::reference("namespace", self.id))?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }
}
