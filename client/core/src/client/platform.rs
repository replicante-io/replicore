//! Implement the namespace methods for API clients.
use anyhow::Context;
use anyhow::Result;

use replisdk::core::models::platform::Platform;

use super::Client;
use crate::error::EmptyResponse;
use crate::error::ResourceIdentifier;

/// Access platform operations.
pub struct PlatformClient<'a> {
    inner: &'a Client,
    name: &'a str,
    ns_id: &'a str,
}

impl Client {
    /// Namespace operations.
    pub fn platform<'a>(&'a self, ns_id: &'a str, name: &'a str) -> PlatformClient<'a> {
        PlatformClient {
            inner: self,
            name,
            ns_id,
        }
    }
}

impl<'a> PlatformClient<'a> {
    /// Delete a [`Platform`] object from the control plane.
    pub async fn delete(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/platform/{}/{}",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.delete(url).send().await?;
        crate::error::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("platform", id)
            })?;
        Ok(())
    }

    /// Schedule a background discovery task for a [`Platform`].
    pub async fn discover(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/platform/{}/{}/discover",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        crate::error::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("platform", id)
            })?;
        Ok(())
    }

    /// Fetch a [`Platform`] record from the server.
    pub async fn get(&'a self) -> Result<Platform> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/platform/{}/{}",
            self.inner.base, self.ns_id, self.name,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = crate::error::inspect::<Platform>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}", self.ns_id, self.name);
                ResourceIdentifier::reference("platform", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }
}
