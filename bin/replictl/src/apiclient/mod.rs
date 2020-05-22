use anyhow::Context as _;
use anyhow::Result;
use slog::debug;
use slog::Logger;
use uuid::Uuid;

use replicante_models_core::api::apply::ApplyObject;

use crate::context::Context;

mod http;

const ENDPOINT_APPLY: &str = "/api/unstable/core/apply";
const ENDPOINT_CLUSTER: &str = "/api/unstable/core/cluster";
const ENDPOINT_CLUSTER_ACTION: &str = "action";
const ENDPOINT_CLUSTER_ACTION_APPROVE: &str = "approve";
const ENDPOINT_CLUSTER_ACTION_DISAPPROVE: &str = "disapprove";
const ENDPOINT_CLUSTER_REFRESH: &str = "refresh";

/// Replicante Core API client.
pub struct RepliClient {
    client: http::HttpClient,
    logger: Logger,
}

impl RepliClient {
    /// Approve a PENDING_APPROVE action so it can be scheduled.
    pub async fn action_approve(&self, cluster: &str, action: Uuid) -> Result<()> {
        debug!(
            self.logger, "About to POST action approve request";
            "action" => %action,
            "cluster" => cluster,
        );
        let uri = format!(
            "{}/{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION,
            action,
            ENDPOINT_CLUSTER_ACTION_APPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Unable to approve action")?;
        response.check_status()?;
        Ok(())
    }

    /// Dispprove a PENDING_APPROVE action so it will not be scheduled.
    pub async fn action_disapprove(&self, cluster: &str, action: Uuid) -> Result<()> {
        debug!(
            self.logger, "About to POST action disapprove request";
            "action" => %action,
            "cluster" => cluster,
        );
        let uri = format!(
            "{}/{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION,
            action,
            ENDPOINT_CLUSTER_ACTION_DISAPPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Unable to disapprove action")?;
        response.check_status()?;
        Ok(())
    }

    /// Send an `ApplyObject` to Replicate Core to request changes.
    pub async fn apply(&self, object: ApplyObject) -> Result<serde_json::Value> {
        debug!(self.logger, "About to POST apply request"; "object" => ?object);
        let request = self.client.post(ENDPOINT_APPLY).json(&object);
        let response = self
            .client
            .send(request)
            .await
            .context("Unable to apply object")?;

        // Check apply-specific errors.
        if response.status().as_u16() == 400 {
            let remote = response
                .body_as()
                .context("Failed to decode apply validation errors from API server")?;
            anyhow::bail!(crate::InvalidApply::new(remote));
        }
        response.check_status()?;

        // Decode and return response payload on success.
        let remote = response.into_body();
        debug!(
            self.logger,
            "Recevied success response from apply API";
            "response" => ?remote
        );
        Ok(remote)
    }

    /// Schedule a refresh task for the given cluster.
    pub async fn cluster_refresh(&self, cluster: &str) -> Result<()> {
        debug!(self.logger, "About to POST cluster refresh request"; "cluster" => cluster);
        let uri = format!(
            "{}/{}/{}",
            ENDPOINT_CLUSTER, cluster, ENDPOINT_CLUSTER_REFRESH,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to schedule the cluster refresh")?;
        response.check_status()?;
        Ok(())
    }

    /// Instantiate a new Replicante API client with the given session.
    pub async fn new(logger: &Logger, context: Context) -> Result<RepliClient> {
        let client = http::HttpClient::new(logger, &context).await?;
        let logger = logger.clone();
        Ok(RepliClient { client, logger })
    }
}

/// Return in case of an API 404 response.
#[derive(thiserror::Error, Debug)]
#[error("API resource not found")]
pub struct ApiNotFound;
