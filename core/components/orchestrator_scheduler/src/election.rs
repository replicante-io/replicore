use humthreads::ThreadScope;
use slog::debug;
use slog::trace;
use slog::Logger;

use replicante_service_coordinator::Election as BaseElection;
use replicante_service_coordinator::Error as CoordinatorError;
use replicante_service_coordinator::LoopingElectionControl;
use replicante_service_coordinator::LoopingElectionLogic;
use replicante_service_coordinator::Result as CoordinatorResult;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use super::logic::Logic;
use super::metrics::DURATION;
use super::metrics::LOOP_COUNT;
use super::metrics::LOOP_ERRORS;

/// Looping election implementation to call into the `Logic`.
pub struct Election {
    logger: Logger,
    logic: Logic,
    thread: ThreadScope,
}

impl Election {
    pub fn new(logic: Logic, logger: Logger, thread: ThreadScope) -> Election {
        Election {
            logger,
            logic,
            thread,
        }
    }
}

impl LoopingElectionLogic for Election {
    fn handle_error(&self, error: CoordinatorError) -> LoopingElectionControl {
        capture_fail!(&error, self.logger, "Orchestrator election error"; failure_info(&error));
        LoopingElectionControl::Continue
    }

    fn post_check(&self, election: &BaseElection) -> CoordinatorResult<LoopingElectionControl> {
        self.thread
            .activity(format!("(idle) election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn pre_check(&self, election: &BaseElection) -> CoordinatorResult<LoopingElectionControl> {
        self.thread
            .activity(format!("election status: {:?}", election.status()));
        Ok(LoopingElectionControl::Proceed)
    }

    fn primary(&self, _: &BaseElection) -> CoordinatorResult<LoopingElectionControl> {
        let _activity = self
            .thread
            .scoped_activity("scheduling pending ClusterSettings orchestrations");
        LOOP_COUNT.inc();
        let timer = DURATION.start_timer();
        trace!(
            self.logger,
            "Started pending ClusterSettings orchestrations cycle",
        );
        if let Err(error) = self.logic.run() {
            LOOP_ERRORS.inc();
            capture_fail!(
                &error,
                self.logger,
                "Unable to schedule pending ClusterSettings orchestrations";
                failure_info(&error),
            );
            return Ok(LoopingElectionControl::Proceed);
        }
        timer.observe_duration();
        trace!(
            self.logger,
            "Pending ClusterSettings orchestrations cycle finished",
        );
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &BaseElection) -> CoordinatorResult<LoopingElectionControl> {
        debug!(self.logger, "Orchestrator election is secondary");
        Ok(LoopingElectionControl::Proceed)
    }
}
