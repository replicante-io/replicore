use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

use error_chain::ChainedError;

use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;
use prometheus::Registry;

use slog::Logger;

use replicante_agent_client::Client;
use replicante_agent_client::HttpClient;
use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::Discovery;
use replicante_agent_discovery::discover;

use replicante_data_models::Node;
use replicante_data_store::Store;

use super::Interfaces;
use super::Result;


lazy_static! {
    /// Counter for discovery cycles.
    static ref DISCOVERY_COUNT: Counter = Counter::with_opts(
        Opts::new("replicante_discovery_loops", "Number of discovery runs started")
    ).expect("Failed to create DISCOVERY_COUNT counter");

    /// Counter for discovery cycles that fail to fetch agents.
    static ref DISCOVERY_FETCH_ERRORS_COUNT: Counter = Counter::with_opts(
        Opts::new("replicante_discovery_fetch_errors", "Number of errors during agent discovery")
    ).expect("Failed to create DISCOVERY_FETCH_ERRORS_COUNT counter");

    /// Counter for discovery cycles that fail to process agents.
    static ref DISCOVERY_PROCESS_ERRORS_COUNT: Counter = Counter::with_opts(
        Opts::new(
            "replicante_discovery_process_errors",
            "Number of errors during processing of discovered agents"
        )
    ).expect("Failed to create DISCOVERY_PROCESS_ERRORS_COUNT counter");

    /// Observe duration of agent discovery.
    static ref DISCOVERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "replicante_discovery_duration",
            "Duration (in seconds) of agent discovery runs"
        ).buckets(vec![0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0])
    ).expect("Failed to create DISCOVERY_DURATION histogram");
}


/// Attemps to register metrics with the Repositoy.
///
/// Metrics that fail to register are logged and ignored.
fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(DISCOVERY_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_FETCH_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_FETCH_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_PROCESS_ERRORS_COUNT.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_PROCESS_ERRORS_COUNT"; "error" => error);
    }
    if let Err(err) = registry.register(Box::new(DISCOVERY_DURATION.clone())) {
        let error = format!("{:?}", err);
        debug!(logger, "Failed to register DISCOVERY_DURATION"; "error" => error);
    }
}


/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Discovery backends configuration.
    #[serde(default)]
    pub backends: BackendsConfig,

    /// Seconds to wait between discovery runs.
    #[serde(default = "Config::default_interval")]
    pub interval: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backends: BackendsConfig::default(),
            interval: Config::default_interval(),
        }
    }
}

impl Config {
    /// Default value for `interval` used by serde.
    fn default_interval() -> u64 { 60 }
}


/// Components to periodically perform service discovery.
pub struct DiscoveryComponent {
    config: BackendsConfig,
    interval: Duration,
    logger: Logger,
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
            store: interfaces.store.clone(),
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let interval = self.interval.clone();
        let worker = DiscoveryWorker::new(
            self.config.clone(),
            self.logger.clone(),
            self.store.clone(),
        );

        info!(self.logger, "Starting Agent Discovery thread");
        let logger = self.logger.clone();
        let thread = ThreadBuilder::new().name(String::from("Agent Discovery")).spawn(move || {
            loop {
                DISCOVERY_COUNT.inc();
                let timer = DISCOVERY_DURATION.start_timer();
                if let Err(err) = worker.run() {
                    let error = err.display_chain().to_string();
                    error!(logger, "Agent discovery iteration failed"; "error" => error);
                }
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


/// Implements the discovery logic of a signle discovery loop.
struct DiscoveryWorker {
    config: BackendsConfig,
    logger: Logger,
    store: Store,
}

impl DiscoveryWorker {
    /// Creates a discover worker.
    pub fn new(config: BackendsConfig, logger: Logger, store: Store) -> DiscoveryWorker {
        DiscoveryWorker {
            config,
            logger,
            store,
        }
    }

    /// Runs a signle discovery loop.
    pub fn run(&self) -> Result<()> {
        debug!(self.logger, "Discovering agents ...");
        for agent in discover(self.config.clone())? {
            let agent = match agent {
                Ok(agent) => agent,
                Err(err) => {
                    let error = err.display_chain().to_string();
                    error!(self.logger, "Failed to fetch agent"; "error" => error);
                    DISCOVERY_FETCH_ERRORS_COUNT.inc();
                    continue;
                }
            };
            if let Err(err) = self.process(agent) {
                let error = err.display_chain().to_string();
                error!(self.logger, "Failed to process agent"; "error" => error);
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
            }
        }
        debug!(self.logger, "Agents discovery complete");
        Ok(())
    }

    /// Process a discovery result to fetch the node state.
    // TODO: replace with use of tasks (one task per discovery?)
    fn process(&self, discovery: Discovery) -> Result<()> {
        let expect_cluster = discovery.cluster().clone();
        let node = fetch_state(discovery)?;
        if node.info.datastore.cluster != expect_cluster {
            return Err("Reported cluster does not match expected cluster".into());
        }
        let old = self.store.persist_node(node.clone())?;
        // TODO: figure out if the node changed.
        debug!(self.logger, "Discovered agent state *** Before: {:?} *** After: {:?}", old, node);
        Ok(())
    }
}


/*** NOTE: the code below will likely be moved when tasks are introduced  ***/
/// Converts an agent discovery result into a Node's status.
fn fetch_state(discovery: Discovery) -> Result<Node> {
    let client = HttpClient::new(discovery.target().clone())?;
    let info = client.info()?;
    let status = client.status()?;
    Ok(Node::new(info, status))
}
