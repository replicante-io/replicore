//! Implement the Node Action methods for API clients.
use anyhow::Context;
use anyhow::Result;
use uuid::Uuid;

use replisdk::core::models::naction::NAction;

use repliclient_utils::EmptyResponse;
use repliclient_utils::ResourceIdentifier;

use super::Client;

/// Access NAction operations.
pub struct NActionClient<'a> {
    inner: &'a Client,
    ns_id: &'a str,
    cluster_id: &'a str,
    node_id: &'a str,
    action_id: Uuid,
}

impl Client {
    /// Node Action operations.
    pub fn naction<'a>(
        &'a self,
        ns_id: &'a str,
        cluster_id: &'a str,
        node_id: &'a str,
        action_id: Uuid,
    ) -> NActionClient<'a> {
        NActionClient {
            inner: self,
            ns_id,
            cluster_id,
            node_id,
            action_id,
        }
    }
}

impl<'a> NActionClient<'a> {
    /// Approve a [`NAction`] for scheduling.
    pub async fn approve(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/naction/{}/{}/{}/{}/approve",
            self.inner.base, self.ns_id, self.cluster_id, self.node_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!(
                    "{}.{}/{}/{}",
                    self.ns_id, self.cluster_id, self.node_id, self.action_id,
                );
                ResourceIdentifier::reference("naction", id)
            })?;
        Ok(())
    }

    /// Cancel a [`NAction`] and prevent any further execution.
    pub async fn cancel(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/naction/{}/{}/{}/{}/cancel",
            self.inner.base, self.ns_id, self.cluster_id, self.node_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!(
                    "{}.{}/{}/{}",
                    self.ns_id, self.cluster_id, self.node_id, self.action_id,
                );
                ResourceIdentifier::reference("naction", id)
            })?;
        Ok(())
    }

    /// Fetch a [`NAction`] record from the server.
    pub async fn get(&'a self) -> Result<NAction> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/naction/{}/{}/{}/{}",
            self.inner.base, self.ns_id, self.cluster_id, self.node_id, self.action_id,
        );
        let response = self.inner.client.get(url).send().await?;
        let response = repliclient_utils::inspect::<NAction>(response)
            .await
            .with_context(|| {
                let id = format!(
                    "{}.{}/{}/{}",
                    self.ns_id, self.cluster_id, self.node_id, self.action_id,
                );
                ResourceIdentifier::reference("naction", id)
            })?;
        let response = response.ok_or(EmptyResponse)?;
        Ok(response)
    }

    /// Reject a [`NAction`] to prevent scheduling.
    pub async fn reject(&'a self) -> Result<()> {
        let url = format!(
            "{}api/v0/object/replicante.io/v0/naction/{}/{}/{}/{}/reject",
            self.inner.base, self.ns_id, self.cluster_id, self.node_id, self.action_id,
        );
        let response = self.inner.client.post(url).send().await?;
        repliclient_utils::inspect::<serde_json::Value>(response)
            .await
            .with_context(|| {
                let id = format!(
                    "{}.{}/{}/{}",
                    self.ns_id, self.cluster_id, self.node_id, self.action_id,
                );
                ResourceIdentifier::reference("naction", id)
            })?;
        Ok(())
    }
}
