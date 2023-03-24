use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use replisdk::platform::framework::IPlatform;
use replisdk::platform::models::ClusterDiscoveryResponse;
use replisdk::platform::models::NodeDeprovisionRequest;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;
use replisdk_experimental::platform::templates::TemplateLookup;
use slog::Logger;

use crate::settings::paths::Paths;
use crate::settings::paths::PlayPod;

pub mod node_list;
pub mod node_start;

mod discover;
mod provision;
mod templates;

pub use self::templates::TemplateLoader;

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
    type Context = PlatformContext;

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
        context: &Self::Context,
        request: NodeProvisionRequest,
    ) -> Result<NodeProvisionResponse> {
        self::provision::provision(self, context, request).await
    }
}

pub struct PlatformContext {
    /// Contextual logger to be used by the operation.
    pub logger: Logger,

    /// Template lookup rules.
    pub templates: Arc<TemplateLookup<TemplateLoader>>,
}

impl actix_web::FromRequest for PlatformContext {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self, Self::Error>>>>;

    fn from_request(
        request: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let default = replisdk::platform::framework::DefaultContext::from_request(request, payload);
        let templates = request
            .app_data::<actix_web::web::Data<TemplateLookup<TemplateLoader>>>()
            .map(|data| data.clone().into_inner())
            .expect("no TemplateLookup<TemplateLoader> attached to actix-web App");
        Box::pin(async move {
            let default = default.await?;
            let context = PlatformContext {
                logger: default.logger,
                templates,
            };
            Ok(context)
        })
    }
}
