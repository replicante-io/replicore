use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

use prometheus::Registry;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_data_store::Store;

use super::Interfaces;
use super::Result;


mod config;
mod metrics;
mod worker;

pub use self::config::Config;

use self::metrics::DISCOVERY_COUNT;
use self::metrics::DISCOVERY_DURATION;
use self::metrics::register_metrics;

use self::worker::DiscoveryWorker;


/// Component to periodically perform service discovery.
pub struct DiscoveryComponent {
    config: BackendsConfig,
    interval: Duration,
    logger: Logger,
    registry: Registry,
    store: Store,

    worker: Option<JoinHandle<()>>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(config: Config, logger: Logger, interfaces: &Interfaces) -> DiscoveryComponent {
        let interval = Duration::from_secs(config.interval);
        register_metrics(&logger, interfaces.metrics.registry());
        DiscoveryComponent {
            config: config.backends,
            interval,
            logger,
            registry: interfaces.metrics.registry().clone(),
            store: interfaces.store.clone(),
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let interval = self.interval.clone();
        DiscoveryWorker::register_metrics(&self.logger, &self.registry);
        let worker = DiscoveryWorker::new(
            self.config.clone(),
            self.logger.clone(),
            self.store.clone(),
        );

        info!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new().name(String::from("Agent Discovery")).spawn(move || {
            loop {
                DISCOVERY_COUNT.inc();
                let timer = DISCOVERY_DURATION.start_timer();
                worker.run();
                timer.observe_duration();
                sleep(interval.clone());
            }
        })?;
        self.worker = Some(thread);
        Ok(())
    }

    /// Wait for the worker thread to stop.
    pub fn wait(&mut self) -> Result<()> {
        info!(self.logger, "Waiting for Agent Discovery to stop");
        self.worker.take().map(|handle| handle.join());
        Ok(())
    }
}
