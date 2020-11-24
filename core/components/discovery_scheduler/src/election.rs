use humthreads::ThreadScope;
use slog::debug;
use slog::trace;
use slog::Logger;

use replicante_service_coordinator::Election;
use replicante_service_coordinator::Error as CoordinatorError;
use replicante_service_coordinator::LoopingElectionControl;
use replicante_service_coordinator::LoopingElectionLogic;
use replicante_service_coordinator::Result as CoordinatorResult;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use super::logic::DiscoveryLogic;
use super::metrics::DURATION;
use super::metrics::LOOP_COUNT;
use super::metrics::LOOP_ERRORS;

/// Looping election implementation to call into the `DiscoveryLogic`.
pub struct DiscoveryElection {
    logger: Logger,
    logic: DiscoveryLogic,
    thread: ThreadScope,
}

impl DiscoveryElection {
    pub fn new(logic: DiscoveryLogic, logger: Logger, thread: ThreadScope) -> DiscoveryElection {
        DiscoveryElection {
            logic,
            logger,
            thread,
        }
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
        let _activity = self
            .thread
            .scoped_activity("scheduling pending discovery runs");
        LOOP_COUNT.inc();
        let timer = DURATION.start_timer();
        trace!(self.logger, "Started pending discovery runs cycle");
        if let Err(error) = self.logic.run() {
            LOOP_ERRORS.inc();
            capture_fail!(
                &error,
                self.logger,
                "Unable to schedule pending discovery runs";
                failure_info(&error),
            );
            return Ok(LoopingElectionControl::Proceed);
        }
        timer.observe_duration();
        trace!(self.logger, "Pending discovery runs cycle finished");
        Ok(LoopingElectionControl::Proceed)
    }

    fn secondary(&self, _: &Election) -> CoordinatorResult<LoopingElectionControl> {
        debug!(self.logger, "Discovery election is secondary");
        Ok(LoopingElectionControl::Proceed)
    }
}
