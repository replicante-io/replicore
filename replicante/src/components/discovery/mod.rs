use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::time::Duration;

use failure::ResultExt;
use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_coordinator::LoopingElection;
use replicante_coordinator::LoopingElectionOpts;

use super::super::Error;
use super::super::ErrorKind;
use super::super::tasks::Tasks;
use super::Interfaces;
use super::Result;


mod config;
mod election;
mod metrics;


pub use self::config::Config;
pub use self::metrics::register_metrics;
use self::election::DiscoveryElection;


/// Component to periodically perform service discovery.
pub struct DiscoveryComponent {
    config: Config,
    coordinator: Coordinator,
    interval: Duration,
    logger: Logger,
    tasks: Tasks,
    worker: Option<JoinHandle<()>>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(
        discovery_config: Config, logger: Logger, interfaces: &Interfaces
    ) -> DiscoveryComponent {
        let interval = Duration::from_secs(discovery_config.interval);
        DiscoveryComponent {
            config: discovery_config,
            coordinator: interfaces.coordinator.clone(),
            interval,
            logger,
            tasks: interfaces.tasks.clone(),
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let config = self.config.backends.clone();
        let coordinator = self.coordinator.clone();
        let interval = self.interval;
        let logger = self.logger.clone();
        let tasks = self.tasks.clone();
        let term = self.config.term;

        info!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new().name("r:c:discovery".into()).spawn(move || {
            let election = coordinator.election("discovery");
            let logic = DiscoveryElection::new(config, logger.clone(), tasks);
            let opts = LoopingElectionOpts::new(election, logic)
                .loop_delay(interval);
            let opts = match term {
                0 => opts,
                term => opts.election_term(term),
            };
            let mut election = LoopingElection::new(opts, logger);
            election.loop_forever();
        }).context(ErrorKind::SpawnThread("agent discovery"))?;
        self.worker = Some(thread);
        Ok(())
    }

    /// Wait for the worker thread to stop.
    pub fn wait(&mut self) -> Result<()> {
        info!(self.logger, "Waiting for Agent Discovery to stop");
        if let Some(handle) = self.worker.take() {
            if let Err(error) = handle.join() {
                let error: Error = format_err!("discovery thread failed: {:?}", error).into();
                return Err(error);
            }
        }
        Ok(())
    }
}
