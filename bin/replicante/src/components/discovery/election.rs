use std::sync::Arc;

use humthreads::ThreadScope;
use opentracingrust::Tracer;
use slog::debug;
use slog::info;
use slog::Logger;

use replicante_cluster_discovery::discover;
use replicante_cluster_discovery::Config as DiscoveryConfig;
use replicante_coordinator::Election;
use replicante_coordinator::Error as CoordinatorError;
use replicante_coordinator::LoopingElectionControl;
use replicante_coordinator::LoopingElectionLogic;
use replicante_coordinator::Result as CoordinatorResult;
use replicante_data_models::ClusterDiscovery;
use replicante_tasks::TaskRequest;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use super::super::super::config::EventsSnapshotsConfig;
use super::super::super::task_payload::ClusterRefreshPayload;
use super::super::super::tasks::ReplicanteQueues;
use super::super::super::tasks::Tasks;
use super::metrics::DISCOVERY_COUNT;
use super::metrics::DISCOVERY_DURATION;
use super::metrics::DISCOVERY_LOOP_ERRORS;
use super::snapshot::EmissionTracker;

/// Main discovery logic with primary/secondaries HA support.
pub struct DiscoveryElection {
    discovery_config: DiscoveryConfig,
    emissions: EmissionTracker,
    logger: Logger,
    tasks: Tasks,
    thread: ThreadScope,
    tracer: Arc<Tracer>,
}

impl DiscoveryElection {
    pub fn new(
        discovery_config: DiscoveryConfig,
        snapshots_config: EventsSnapshotsConfig,
        logger: Logger,
        tasks: Tasks,
        thread: ThreadScope,
        tracer: Arc<Tracer>,
    ) -> DiscoveryElection {
        DiscoveryElection {
            discovery_config,
            emissions: EmissionTracker::new(snapshots_config),
            logger,
            tasks,
            thread,
            tracer,
        }
    }
}

impl DiscoveryElection {
    /// Emit a cluster refresh task for the discovery.
    fn emit(&self, cluster: ClusterDiscovery) {
        let cluster_id = cluster.cluster_id.clone();
        let snapshot = self.emissions.snapshot(cluster_id.clone());
        let payload = ClusterRefreshPayload::new(cluster, snapshot);
        let mut task = TaskRequest::new(ReplicanteQueues::ClusterRefresh);
        let span = self.tracer.span("cluster.discovery").auto_finish();
        if let Err(error) = task.trace(span.context(), &self.tracer) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context in task request";
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        }
        if let Err(error) = self.tasks.request(task, payload) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to request cluster refresh";
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        };
    }
}

impl LoopingElectionLogic for DiscoveryElection {
    fn handle_error(&self, error: CoordinatorError) -> LoopingElectionControl {
        capture_fail!(&error, self.logger, "Discovery election error"; failure_info(&error));
        LoopingElectionControl::Continue
    }

    fn post_check(&self, election: &Election) -> CoordinatorResult<LoopingElectionControl> {
        self.thread
            .activity(format!("(idle) election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn pre_check(&self, election: &Election) -> CoordinatorResult<LoopingElectionControl> {
        self.thread
            .activity(format!("election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn primary(&self, _: &Election) -> CoordinatorResult<LoopingElectionControl> {
        DISCOVERY_COUNT.inc();
        debug!(self.logger, "Discovering agents ...");
        let _activity = self.thread.scoped_activity("discovering agents");
        let _timer = DISCOVERY_DURATION.start_timer();
        for cluster in discover(self.discovery_config.clone()) {
            match cluster {
                Ok(cluster) => self.emit(cluster),
                Err(error) => {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Cluster discovery error";
                        failure_info(&error),
                    );
                    DISCOVERY_LOOP_ERRORS.inc();
                }
            }
        }
        info!(self.logger, "Agents discovery complete");
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &Election) -> CoordinatorResult<LoopingElectionControl> {
        debug!(self.logger, "Discovery election is secondary");
        self.emissions.reset();
        Ok(LoopingElectionControl::Proceed)
    }
}
