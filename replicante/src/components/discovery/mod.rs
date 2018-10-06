use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

use prometheus::Registry;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

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
    agents_api_timeout: Duration,
    config: BackendsConfig,
    events: EventsStream,
    interval: Duration,
    logger: Logger,
    registry: Registry,
    store: Store,

    worker: Option<JoinHandle<()>>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(
        config: Config, agents_api_timeout: Duration, logger: Logger, interfaces: &Interfaces
    ) -> DiscoveryComponent {
        let interval = Duration::from_secs(config.interval);
        register_metrics(&logger, interfaces.metrics.registry());
        DiscoveryComponent {
            agents_api_timeout,
            config: config.backends,
            events: interfaces.streams.events.clone(),
            interval,
            logger,
            registry: interfaces.metrics.registry().clone(),
            store: interfaces.store.clone(),
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let interval = self.interval;
        DiscoveryWorker::register_metrics(&self.logger, &self.registry);
        let worker = DiscoveryWorker::new(
            self.config.clone(),
            self.logger.clone(),
            self.events.clone(),
            self.store.clone(),
            self.agents_api_timeout.clone(),
        );

        info!(self.logger, "Starting Agent Discovery thread");
        let thread = ThreadBuilder::new().name(String::from("Agent Discovery")).spawn(move || {
            loop {
                DISCOVERY_COUNT.inc();
                let timer = DISCOVERY_DURATION.start_timer();
                worker.run();
                timer.observe_duration();
                sleep(interval);
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
