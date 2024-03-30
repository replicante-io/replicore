//! Implement the orchestrator Action methods for API clients.
use anyhow::Context;
use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::oaction::OAction;

use super::Client;
use crate::error::EmptyResponse;
use crate::error::ResourceIdentifier;

/// Access OAction operations.
pub struct OActionClient<'a> {
    inner: &'a Client,
    ns_id: &'a str,
    cluster_id: &'a str,
    action_id: Uuid,
}

impl Client {
    /// Orchestrator Action operations.
    pub fn oaction<'a>(
        &'a self,
        ns_id: &'a str,
        cluster_id: &'a str,
        action_id: Uuid,
    ) -> OActionClient<'a> {
        OActionClient {
            inner: self,
            ns_id,
            cluster_id,
            action_id,
        }
    }
}

impl<'a> OActionClient<'a> {
    /// Fetch a [`OAction`] record from the server.
    pub async fn get(&'a self) -> Result<OAction> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/oaction/{}/{}/{}",
            self.inner.base, self.ns_id, self.cluster_id, self.action_id,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = crate::error::inspect::<OAction>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}/{}", self.ns_id, self.cluster_id, self.action_id);
                ResourceIdentifier::reference("oaction", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }
}
