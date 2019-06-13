use std::sync::Arc;

use opentracingrust::Tracer;
use slog::Logger;

use replicante_models_api::HealthStatus;
use replicante_service_healthcheck::HealthCheck;
use replicante_service_healthcheck::HealthChecks;

use super::super::super::config::ZookeeperConfig;
use super::super::super::coordinator::Election;
use super::super::super::coordinator::NonBlockingLock;
use super::super::super::NodeId;
use super::super::super::Result;
use super::super::Backend;
use super::client::Client;

mod cleaner;
mod election;
mod lock;

use self::cleaner::Cleaner;

/// Zookeeper-backed distributed coordination.
pub struct Zookeeper {
    // Background thread to clean unused nodes.
    _cleaner: Cleaner,
    client: Arc<Client>,
    logger: Logger,
    node_id: NodeId,
    tracer: Option<Arc<Tracer>>,
}

impl Zookeeper {
    pub fn new<T>(
        node_id: NodeId,
        config: ZookeeperConfig,
        logger: Logger,
        healthchecks: &mut HealthChecks,
        tracer: T,
    ) -> Result<Zookeeper>
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let client = Arc::new(Client::new(config.clone(), Some(&node_id), logger.clone())?);
        let cleaner = Cleaner::new(Arc::clone(&client), config, node_id.clone(), logger.clone())?;
        let tracer = tracer.into();
        let healthcheck = ZookeeperHealthCheck {
            client: Arc::clone(&client),
        };
        healthchecks.register("coordination", healthcheck);
        Ok(Zookeeper {
            _cleaner: cleaner,
            client,
            logger,
            node_id,
            tracer,
        })
    }
}

impl Backend for Zookeeper {
    fn election(&self, id: String) -> Election {
        Election::new(
            id.clone(),
            Box::new(self::election::ZookeeperElection::new(
                Arc::clone(&self.client),
                &id,
                self.node_id.clone(),
                self.logger.clone(),
            )),
        )
    }

    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock {
        NonBlockingLock::new(Box::new(self::lock::ZookeeperNBLock::new(
            Arc::clone(&self.client),
            lock,
            self.node_id.clone(),
            self.logger.clone(),
            self.tracer.clone(),
        )))
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}

/// Check that the current session is active.
struct ZookeeperHealthCheck {
    client: Arc<Client>,
}

impl HealthCheck for ZookeeperHealthCheck {
    fn check(&self) -> HealthStatus {
        if self.client.is_connected() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Failed("client not connected to zookeeper".to_string())
        }
    }
}
