//! Implement the orchestrator Action methods for API clients.
use anyhow::Context;
use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::oaction::OAction;

use repliclient_utils::EmptyResponse;
use repliclient_utils::ResourceIdentifier;

use super::Client;

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
    /// Approve an [`OAction`] for scheduling.
    pub async fn approve(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/oaction/{}/{}/{}/approve",
            self.inner.base, self.ns_id, self.cluster_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}/{}", self.ns_id, self.cluster_id, self.action_id);
                ResourceIdentifier::reference("oaction", id)
            })?;
        Ok(())
    }

    /// Cancel an [`OAction`] and prevent any further execution.
    pub async fn cancel(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/oaction/{}/{}/{}/cancel",
            self.inner.base, self.ns_id, self.cluster_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}/{}", self.ns_id, self.cluster_id, self.action_id);
                ResourceIdentifier::reference("oaction", id)
            })?;
        Ok(())
    }

    /// Fetch an [`OAction`] record from the server.
    pub async fn get(&'a self) -> Result<OAction> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/oaction/{}/{}/{}",
            self.inner.base, self.ns_id, self.cluster_id, self.action_id,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<OAction>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}/{}", self.ns_id, self.cluster_id, self.action_id);
                ResourceIdentifier::reference("oaction", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }

    /// Reject an [`OAction`] to prevent scheduling.
    pub async fn reject(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/oaction/{}/{}/{}/reject",
            self.inner.base, self.ns_id, self.cluster_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!("{}.{}/{}", self.ns_id, self.cluster_id, self.action_id);
                ResourceIdentifier::reference("oaction", id)
            })?;
        Ok(())
    }
}
