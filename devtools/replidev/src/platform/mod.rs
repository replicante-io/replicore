use anyhow::Result;
use replisdk::platform::framework::IPlatform;
use replisdk::platform::models::ClusterDiscoveryResponse;
use replisdk::platform::models::NodeDeprovisionRequest;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;

use crate::settings::paths::Paths;
use crate::settings::paths::PlayPod;

pub mod node_list;
pub mod node_start;

mod discover;
mod provision;
mod templates;

/// Manage data store nodes using podman and Replicante's playground templates.
pub struct Platform {
    /// Address to present the agents as reachable on.
    agents_address: String,

    /// Full configuration for the process.
    conf: crate::Conf,
}

impl Platform {
    pub fn from_conf(conf: crate::Conf) -> Platform {
        let agents_address = conf.resolve_play_server_agents_address();
        Platform { agents_address, conf }
    }
}

#[async_trait::async_trait]
impl IPlatform for Platform {
    type Context = replisdk::platform::framework::DefaultContext;

    async fn deprovision(
        &self,
        _context: &Self::Context,
        request: NodeDeprovisionRequest,
    ) -> Result<()> {
        // Stop the node pod.
        crate::podman::pod_stop(&self.conf, &request.node_id)
            .await
            .map_err(replisdk::utils::actix::error::Error::from)?;

        // Once the pod node is stopped its data can be deleted.
        let paths = PlayPod::new("<deprovision>", &request.cluster_id, &request.node_id);
        let data = paths.data();
        crate::podman::unshare(&self.conf, vec!["rm", "-r", data]).await?;
        Ok(())
    }

    async fn discover(&self, _context: &Self::Context) -> Result<ClusterDiscoveryResponse> {
        self::discover::discover(self).await
    }

    async fn provision(
        &self,
        _context: &Self::Context,
        request: NodeProvisionRequest,
    ) -> Result<NodeProvisionResponse> {
        self::provision::provision(self, request).await
    }
}
