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
use replicante_util_upkeep::Upkeep;

use super::Component;
use super::Interfaces;
use crate::config::EventsSnapshotsConfig;
use crate::tasks::Tasks;
use crate::ErrorKind;
use crate::Result;

mod config;
mod election;
mod metrics;
mod snapshot;

pub use self::config::Config;
pub use self::metrics::register_metrics;

use self::election::DiscoveryElection;

/// Component to periodically perform service discovery.
pub struct DiscoveryComponent {
    config: Config,
    coordinator: Coordinator,
    interval: Duration,
    logger: Logger,
    snapshots_config: EventsSnapshotsConfig,
    tasks: Tasks,
    tracer: Arc<Tracer>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(
        discovery_config: Config,
        snapshots_config: EventsSnapshotsConfig,
        logger: Logger,
        interfaces: &Interfaces,
    ) -> DiscoveryComponent {
        let interval = Duration::from_secs(discovery_config.interval);
        DiscoveryComponent {
            config: discovery_config,
            coordinator: interfaces.coordinator.clone(),
            interval,
            logger,
            snapshots_config,
            tasks: interfaces.tasks.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }
}

impl Component for DiscoveryComponent {
    /// Starts the agent discovery process in a background thread.
    fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let config = self.config.backends.clone();
        let coordinator = self.coordinator.clone();
        let interval = self.interval;
        let logger = self.logger.clone();
        let snapshots_config = self.snapshots_config.clone();
        let tasks = self.tasks.clone();
        let term = self.config.term;
        let tracer = Arc::clone(&self.tracer);
        let (shutdown_sender, shutdown_receiver) = LoopingElectionOpts::shutdown_channel();

        debug!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new("r:c:discovery")
            .full_name("replicore:component:discovery")
            .spawn(move |scope| {
                scope.activity("initialising agent discovery election");
                let election = coordinator.election("discovery");
                let logic = DiscoveryElection::new(
                    config,
                    snapshots_config,
                    logger.clone(),
                    tasks,
                    scope,
                    tracer,
                );
                let opts = LoopingElectionOpts::new(election, logic)
                    .loop_delay(interval)
                    .shutdown_receiver(shutdown_receiver);
                let opts = match term {
                    0 => opts,
                    term => opts.election_term(term),
                };
                let mut election = LoopingElection::new(opts, logger);
                election.loop_forever();
            })
            .with_context(|_| ErrorKind::ThreadSpawn("agent discovery"))?;
        upkeep.on_shutdown(move || shutdown_sender.request());
        upkeep.register_thread(thread);
        Ok(())
    }
}
