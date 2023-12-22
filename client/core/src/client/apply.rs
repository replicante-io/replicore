//! Implement the apply method for API clients.
use anyhow::Result;
use serde_json::Value as Json;

use super::Client;

impl Client {
    /// Apply a manifest to the control plane.
    pub async fn apply(&self, manifest: Json) -> Result<Json> {
        let response = self
            .client
            .post(format!("{}api/v0/apply", self.base))
            .json(&manifest)
            .send()
            .await?;
        let response = crate::error::inspect(response).await?;
        Ok(response.unwrap_or_default())
    }
}
