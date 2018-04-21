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
use replicante_agent_discovery::discover;
use replicante_agent_models::NodeInfo;
use replicante_agent_models::NodeStatus;

use replicante_data_models::ClusterDiscovery;
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


/// Component to periodically perform service discovery.
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
    pub fn run(&self) {
        debug!(self.logger, "Discovering agents ...");
        for cluster in discover(self.config.clone()) {
            let cluster = match cluster {
                Ok(cluster) => cluster,
                Err(err) => {
                    let error = err.display_chain().to_string();
                    error!(self.logger, "Failed to fetch cluster"; "error" => error);
                    DISCOVERY_FETCH_ERRORS_COUNT.inc();
                    continue;
                }
            };
            self.process(cluster);
        }
        debug!(self.logger, "Agents discovery complete");
    }

    /// Process a discovery result to fetch the node state.
    // TODO: refactor code to complete storage improvement
    fn process(&self, cluster: ClusterDiscovery) {
        // Persist cluster first.
        match self.store.persist_discovery(cluster.clone()) {
            Err(error) => {
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to persist cluster";
                    "cluster" => cluster.name.clone(), "error" => error
                );
            },
            Ok(_old) => {
                // TODO: figure out if the cluster changed.
            },
        };

        // Then process each node.
        for node in cluster.nodes.iter() {
            if let Err(err) = self.process_node(&cluster, node) {
                let error = err.display_chain().to_string();
                error!(
                    self.logger, "Failed to process node";
                    "cluster" => cluster.name.clone(), "error" => error,
                    "node" => node.clone()
                );
                DISCOVERY_PROCESS_ERRORS_COUNT.inc();
            }
        }
    }

    /// Fetch the state of individual nodes.
    fn process_node(&self, cluster: &ClusterDiscovery, node: &String) -> Result<()> {
        let expect_cluster = &cluster.name;
        let (info, _) = fetch_state(node)?;
        if &info.datastore.cluster != expect_cluster {
            return Err("Reported cluster does not match expected cluster".into());
        }
        let old = self.store.persist_node(Node::new(info.datastore))?;
        // TODO: figure out if the node changed.
        debug!(self.logger, "Discovered agent state *** Before: {:?} *** After: {:?}", old, node);
        Ok(())
    }
}


/*** NOTE: the code below will likely be moved when tasks are introduced  ***/
/// Converts an agent discovery result into a Node's status.
fn fetch_state(node: &String) -> Result<(NodeInfo, NodeStatus)> {
    let client = HttpClient::new(node.clone())?;
    let info = client.info()?;
    let status = client.status()?;
    Ok((info, status))
}
