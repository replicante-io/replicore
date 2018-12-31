use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

use failure::ResultExt;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_coordinator::Coordinator;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::super::Error;
use super::super::ErrorKind;
use super::super::config::EventsSnapshotsConfig;
use super::super::tasks::Tasks;
use super::Interfaces;
use super::Result;


mod config;
mod metrics;
mod worker;


pub use self::config::Config;
pub use self::metrics::register_metrics;

use self::metrics::DISCOVERY_COUNT;
use self::metrics::DISCOVERY_DURATION;

use self::worker::DiscoveryWorker;


/// Component to periodically perform service discovery.
pub struct DiscoveryComponent {
    agents_api_timeout: Duration,
    coordinator: Coordinator,
    discovery_config: BackendsConfig,
    events: EventsStream,
    interval: Duration,
    logger: Logger,
    snapshots_config: EventsSnapshotsConfig,
    store: Store,
    tasks: Tasks,
    worker: Option<JoinHandle<()>>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(
        discovery_config: Config, snapshots_config: EventsSnapshotsConfig,
        agents_api_timeout: Duration, logger: Logger, interfaces: &Interfaces
    ) -> DiscoveryComponent {
        let interval = Duration::from_secs(discovery_config.interval);
        DiscoveryComponent {
            agents_api_timeout,
            coordinator: interfaces.coordinator.clone(),
            discovery_config: discovery_config.backends,
            events: interfaces.streams.events.clone(),
            interval,
            logger,
            snapshots_config,
            store: interfaces.store.clone(),
            tasks: interfaces.tasks.clone(),
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let coordinator = self.coordinator.clone();
        let interval = self.interval;
        let worker = DiscoveryWorker::new(
            self.discovery_config.clone(),
            self.snapshots_config.clone(),
            self.logger.clone(),
            self.events.clone(),
            self.store.clone(),
            self.tasks.clone(),
            self.agents_api_timeout.clone(),
        );

        info!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new().name("r:c:discovery".into()).spawn(move || {
            let mut election = coordinator.election("discovery");
            loop {
                election.run().expect("TODO: remove after tests");
                println!("~~~ {:?}", election.status());
                DISCOVERY_COUNT.inc();
                let timer = DISCOVERY_DURATION.start_timer();
                worker.run();
                timer.observe_duration();
                sleep(interval);
                election.step_down().expect("TODO: remove after tests");
            }
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
