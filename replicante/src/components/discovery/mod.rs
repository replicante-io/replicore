use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_coordinator::LoopingElection;
use replicante_coordinator::LoopingElectionOpts;
use replicante_util_upkeep::Upkeep;

use super::super::config::EventsSnapshotsConfig;
use super::super::tasks::Tasks;
use super::super::ErrorKind;
use super::super::Result;
use super::Interfaces;

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
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let config = self.config.backends.clone();
        let coordinator = self.coordinator.clone();
        let interval = self.interval;
        let logger = self.logger.clone();
        let snapshots_config = self.snapshots_config.clone();
        let tasks = self.tasks.clone();
        let term = self.config.term;
        let (shutdown_sender, shutdown_receiver) = LoopingElectionOpts::shutdown_channel();

        info!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new("r:c:discovery")
            .full_name("replicore:component:discovery")
            .spawn(move |scope| {
                scope.activity("initialising agent discovery election");
                let election = coordinator.election("discovery");
                let logic =
                    DiscoveryElection::new(config, snapshots_config, logger.clone(), tasks, scope);
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
