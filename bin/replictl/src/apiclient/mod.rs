use anyhow::Context as _;
use anyhow::Result;
use replisdk::core::models::platform::Platform;
use slog::debug;
use slog::Logger;
use uuid::Uuid;

use replicante_models_core::api::apply::ApplyObject;
use replicante_models_core::api::discovery_settings::DiscoverySettingsListResponse;
use replicante_models_core::api::node_action::NodeActionSummariesResponse;
use replicante_models_core::api::node_action::NodeActionSummary;
use replicante_models_core::api::orchestrator_action::OrchestratorActionSummariesResponse;
use replicante_models_core::api::orchestrator_action::OrchestratorActionSummary;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_models_core::scope::Namespace;

use crate::context::Context;

mod http;

const ENDPOINT_APPLY: &str = "/api/unstable/core/apply";

const ENDPOINT_CLUSTER: &str = "/api/unstable/core/cluster";
const ENDPOINT_CLUSTER_ACTION_APPROVE: &str = "approve";
const ENDPOINT_CLUSTER_ACTION_DISAPPROVE: &str = "disapprove";
const ENDPOINT_CLUSTER_ACTION_NODE: &str = "action";
const ENDPOINT_CLUSTER_ACTION_ORCHESTRATOR: &str = "orchestrator-action";
const ENDPOINT_CLUSTER_ACTION_SUMMARY: &str = "summary";
const ENDPOINT_CLUSTER_ORCHESTRATE: &str = "orchestrate";

const ENDPOINT_DISCOVERY_SETTINGS: &str = "/api/unstable/core/discoverysettings";
const ENDPOINT_DISCOVERY_SETTINGS_DELETE: &str = "delete";
const ENDPOINT_DISCOVERY_SETTINGS_LIST: &str = "list";

const ENDPOINT_NAMESPACE: &str = "/api/unstable/core/namespace";
const ENDPOINT_NAMESPACE_LIST: &str = "list";

const ENDPOINT_PLATFORM: &str = "/api/unstable/core/platform";

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
            ENDPOINT_CLUSTER_ACTION_NODE,
            action,
            ENDPOINT_CLUSTER_ACTION_APPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("unable to approve action")?;
        response.check_status()?;
        Ok(())
    }

    /// Disprove a PENDING_APPROVE action so it will not be scheduled.
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
            ENDPOINT_CLUSTER_ACTION_NODE,
            action,
            ENDPOINT_CLUSTER_ACTION_DISAPPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("unable to disapprove action")?;
        response.check_status()?;
        Ok(())
    }

    /// Return summaries of node actions for a cluster.
    pub async fn action_node_summaries(&self, cluster: &str) -> Result<Vec<NodeActionSummary>> {
        let uri = format!(
            "{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION_NODE,
            ENDPOINT_CLUSTER_ACTION_SUMMARY,
        );
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to list NodeActionSummary objects")?;
        response.check_status()?;
        let response = response
            .body_as::<NodeActionSummariesResponse>()
            .context("Failed to decode NodeActionSummary list response")?;
        Ok(response.actions)
    }

    /// Approve a PENDING_APPROVE orchestrator action so it can be scheduled and executed.
    pub async fn action_orchestrator_approve(&self, cluster: &str, action: Uuid) -> Result<()> {
        let uri = format!(
            "{}/{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION_ORCHESTRATOR,
            action,
            ENDPOINT_CLUSTER_ACTION_APPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("unable to approve orchestrator action")?;
        response.check_status()?;
        Ok(())
    }

    /// Disprove a PENDING_APPROVE orchestrator action so it will not be scheduled.
    pub async fn action_orchestrator_disapprove(&self, cluster: &str, action: Uuid) -> Result<()> {
        let uri = format!(
            "{}/{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION_ORCHESTRATOR,
            action,
            ENDPOINT_CLUSTER_ACTION_DISAPPROVE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("unable to disapprove orchestrator action")?;
        response.check_status()?;
        Ok(())
    }

    /// Return summaries of orchestrator actions for a cluster.
    pub async fn action_orchestrator_summaries(
        &self,
        cluster: &str,
    ) -> Result<Vec<OrchestratorActionSummary>> {
        let uri = format!(
            "{}/{}/{}/{}",
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION_ORCHESTRATOR,
            ENDPOINT_CLUSTER_ACTION_SUMMARY,
        );
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to list OrchestratorActionSummary objects")?;
        response.check_status()?;
        let response = response
            .body_as::<OrchestratorActionSummariesResponse>()
            .context("Failed to decode OrchestratorActionSummary list response")?;
        Ok(response.actions)
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
            // Attempt to decode the response as apply errors.
            // If decoding fails assume response is a generic error.
            let remote = response.body_as::<ErrorsCollection>();
            if let Ok(remote) = remote {
                anyhow::bail!(crate::InvalidApply::new(remote));
            }
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

    /// Delete a DiscoverySettings object.
    pub async fn discovery_settings_delete(&self, namespace: &str, name: &str) -> Result<()> {
        let uri = format!(
            "{}/{}/{}/{}",
            ENDPOINT_DISCOVERY_SETTINGS, namespace, name, ENDPOINT_DISCOVERY_SETTINGS_DELETE,
        );
        let request = self.client.delete(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to delete DiscoverySettings object")?;
        response.check_status()?;
        Ok(())
    }

    /// Fetch the list of names for DiscoverySettings objects in the namespace.
    pub async fn discovery_settings_list(&self, namespace: &str) -> Result<Vec<String>> {
        let uri = format!(
            "{}/{}/{}",
            ENDPOINT_DISCOVERY_SETTINGS, namespace, ENDPOINT_DISCOVERY_SETTINGS_LIST,
        );
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to list DiscoverySettings objects")?;
        response.check_status()?;
        let response = response
            .body_as::<DiscoverySettingsListResponse>()
            .context("Failed to decode DiscoverySettings list response")?;
        Ok(response.names)
    }

    /// Schedule an orchestration task for the given cluster.
    pub async fn orchestrate_cluster(&self, cluster: &str) -> Result<()> {
        debug!(self.logger, "About to POST cluster orchestrate request"; "cluster" => cluster);
        let uri = format!(
            "{}/{}/{}",
            ENDPOINT_CLUSTER, cluster, ENDPOINT_CLUSTER_ORCHESTRATE,
        );
        let request = self.client.post(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to schedule cluster orchestration")?;
        response.check_status()?;
        Ok(())
    }

    /// Query a `Namespace` object.
    pub async fn namespace_get(&self, ns_id: &str) -> Result<Namespace> {
        let uri = format!("{}/{}", ENDPOINT_NAMESPACE, ns_id);
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to query Namespace object")?;
        response.check_status()?;
        let response = response
            .body_as::<Namespace>()
            .context("Failed to decode Namespace response")?;
        Ok(response)
    }

    /// List all `Namespace`s in the cluster.
    pub async fn namespace_list(&self) -> Result<Vec<Namespace>> {
        let uri = format!("{}s/{}", ENDPOINT_NAMESPACE, ENDPOINT_NAMESPACE_LIST);
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to list Namespace objects")?;
        response.check_status()?;
        let response = response
            .body_as::<Vec<Namespace>>()
            .context("Failed to decode Namespace list response")?;
        Ok(response)
    }

    /// Instantiate a new Replicante API client with the given session.
    pub async fn new(logger: &Logger, context: Context) -> Result<RepliClient> {
        let client = http::HttpClient::new(logger, &context).await?;
        let logger = logger.clone();
        Ok(RepliClient { client, logger })
    }

    /// Query a `Platform` object.
    pub async fn platform_get(&self, ns_id: &str, platform: &str) -> Result<Platform> {
        let uri = format!("{}/{}/{}", ENDPOINT_PLATFORM, ns_id, platform);
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to query Platform object")?;
        response.check_status()?;
        let response = response
            .body_as::<Platform>()
            .context("Failed to decode Platform response")?;
        Ok(response)
    }

    /// List all `Platform`s in the namespace.
    pub async fn platform_list(&self, ns_id: &str) -> Result<Vec<Platform>> {
        let uri = format!("{}s/{}", ENDPOINT_PLATFORM, ns_id);
        let request = self.client.get(&uri);
        let response = self
            .client
            .send(request)
            .await
            .context("Failed to list Platform objects")?;
        response.check_status()?;
        let response = response
            .body_as::<Vec<Platform>>()
            .context("Failed to decode Platform list response")?;
        Ok(response)
    }
}

/// Return in case of an API 404 response.
#[derive(thiserror::Error, Debug)]
#[error("API resource not found")]
pub struct ApiNotFound;
