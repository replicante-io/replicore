use slog::Logger;

use replicante_agent_discovery::Config as DiscoveryConfig;
use replicante_agent_discovery::discover;
use replicante_coordinator::Election;
use replicante_coordinator::Error as CoordinatorError;
use replicante_coordinator::LoopingElectionControl;
use replicante_coordinator::LoopingElectionLogic;
use replicante_coordinator::Result as CoordinatorResult;
use replicante_data_models::ClusterDiscovery;
use replicante_tasks::TaskRequest;
use replicante_util_failure::failure_info;

use super::super::super::tasks::ReplicanteQueues;
use super::super::super::tasks::Tasks;
use super::metrics::DISCOVERY_COUNT;
use super::metrics::DISCOVERY_DURATION;
use super::metrics::DISCOVERY_ERRORS;


/// Main discovery logic with primary/secondaries HA support.
pub struct DiscoveryElection {
    discovery_config: DiscoveryConfig,
    logger: Logger,
    tasks: Tasks,
}

impl DiscoveryElection {
    pub fn new(
        discovery_config: DiscoveryConfig, logger: Logger, tasks: Tasks
    ) -> DiscoveryElection {
        DiscoveryElection {
            discovery_config,
            logger,
            tasks,
        }
    }
}

impl DiscoveryElection {
    /// Emit a cluster refresh task for the discovery.
    fn emit(&self, cluster: ClusterDiscovery) {
        let name = cluster.cluster.clone();
        let task = TaskRequest::new(ReplicanteQueues::ClusterRefresh);
        if let Err(error) = self.tasks.request(task, cluster) {
            error!(
                self.logger, "Failed to request cluster refresh";
                "cluster" => name,
                "error" => %error
                // TODO: failure_info(&error)
            );
        };
    }
}

impl LoopingElectionLogic for DiscoveryElection {
    fn handle_error(&self, error: CoordinatorError) -> LoopingElectionControl {
        error!(self.logger, "Discovery election error"; failure_info(&error));
        LoopingElectionControl::Continue
    }

    fn primary(&self, _: &Election) -> CoordinatorResult<LoopingElectionControl> {
        DISCOVERY_COUNT.inc();
        debug!(self.logger, "Discovering agents ...");
        let _timer = DISCOVERY_DURATION.start_timer();
        for cluster in discover(self.discovery_config.clone()) {
            match cluster {
                Ok(cluster) => self.emit(cluster),
                Err(error) => {
                    error!(
                        self.logger, "Cluster discovery error";
                        "error" => %error
                        // TODO: failure_info(&error)
                    );
                    DISCOVERY_ERRORS.inc();
                }
            }
        }
        info!(self.logger, "Agents discovery complete");
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &Election) -> CoordinatorResult<LoopingElectionControl> {
        debug!(self.logger, "Discovery election is secondary");
        Ok(LoopingElectionControl::Proceed)
    }
}
