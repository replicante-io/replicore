use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_service_coordinator::Coordinator;
use replicante_service_coordinator::LoopingElection;
use replicante_service_coordinator::LoopingElectionOpts;
use replicante_store_primary::store::Store;
use replicante_util_upkeep::Upkeep;

use replicore_models_tasks::Tasks;

mod config;
mod election;
mod error;
mod logic;
mod metrics;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

const RUN_ALREADY_CALLED: &str = "called OrchestratorScheduler::run more then once";

/// Schedule cluster orchestration tasks waiting to run.
pub struct OrchestratorScheduler {
    coordinator: Option<Coordinator>,
    interval: Duration,
    logger: Logger,
    logic: Option<self::logic::Logic>,
    term: u64,
}

impl OrchestratorScheduler {
    pub fn new(
        coordinator: Coordinator,
        config: Config,
        logger: Logger,
        store: Store,
        tasks: Tasks,
        tracer: Arc<Tracer>,
    ) -> OrchestratorScheduler {
        let coordinator = Some(coordinator);
        let interval = Duration::from_secs(config.interval);
        let logic = self::logic::Logic::new(logger.clone(), store, tasks, tracer);
        let logic = Some(logic);
        OrchestratorScheduler {
            coordinator,
            interval,
            logger,
            logic,
            term: config.term,
        }
    }

    /// Start the component in a background thread and return.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let coordinator = self.coordinator.take().expect(RUN_ALREADY_CALLED);
        let interval = self.interval;
        let logger = self.logger.clone();
        let logic = self.logic.take().expect(RUN_ALREADY_CALLED);
        let term = self.term;
        let (shutdown_sender, shutdown_receiver) = LoopingElectionOpts::shutdown_channel();

        debug!(self.logger, "Starting ClusterSettings scheduler thread");
        let thread = ThreadBuilder::new("r:c:orchestrator")
            .full_name("replicore:component:orchestrator")
            .spawn(move |scope| {
                scope.activity("initialising ClusterSettings scheduler election");
                let election = coordinator.election("orchestrator");
                let looper = self::election::Election::new(logic, logger.clone(), scope);
                let opts = LoopingElectionOpts::new(election, looper)
                    .loop_delay(interval)
                    .shutdown_receiver(shutdown_receiver);
                let opts = match term {
                    0 => opts,
                    term => opts.election_term(term),
                };
                let mut election = LoopingElection::new(opts, logger);
                election.loop_forever();
            })
            .with_context(|_| ErrorKind::ThreadSpawn)?;
        upkeep.on_shutdown(move || shutdown_sender.request());
        upkeep.register_thread(thread);
        Ok(())
    }
}
