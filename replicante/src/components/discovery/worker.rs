use std::time::Duration;
use error_chain::ChainedError;
use prometheus::Registry;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::discover;

use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;
use replicante_data_store::Store;

use super::super::super::Result;
use super::super::super::ResultExt;
use super::metrics::DISCOVERY_FETCH_ERRORS_COUNT;


const FAIL_FIND_DISCOVERY: &str = "Failed to fetch cluster discovery";
const FAIL_PERSIST_DISCOVERY: &str = "Failed to persist cluster discovery";


/// Implements the discovery logic of a signle discovery loop.
pub struct DiscoveryWorker {
    config: BackendsConfig,
    logger: Logger,
    store: Store,

    // TODO(stefano): move into dedicated component when possible.
    aggregator: Aggregator,
    fetcher: Fetcher,
}

impl DiscoveryWorker {
    /// Creates a discover worker.
    pub fn new(
        config: BackendsConfig, logger: Logger, store: Store, timeout: Duration
    ) -> DiscoveryWorker {
        let aggregator = Aggregator::new(logger.clone(), store.clone());
        let fetcher = Fetcher::new(logger.clone(), store.clone(), timeout);
        DiscoveryWorker {
            config,
            logger,
            store,

            // TODO(stefano): move into dedicated component when possible.
            aggregator,
            fetcher,
        }
    }

    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        Aggregator::register_metrics(logger, registry);
        Fetcher::register_metrics(logger, registry);
    }

    /// Runs a signle discovery loop.
    pub fn run(&self) {
        debug!(self.logger, "Discovering agents ...");
        for cluster in discover(self.config.clone()) {
            match cluster {
                Ok(cluster) => self.process(cluster),
                Err(error) => {
                    let error = error.display_chain().to_string();
                    error!(self.logger, "Failed to fetch cluster discovery"; "error" => error);
                    DISCOVERY_FETCH_ERRORS_COUNT.inc();
                }
            };
        }
        debug!(self.logger, "Agents discovery complete");
    }
}

impl DiscoveryWorker {
    /// Process a discovery result to fetch the node state.
    ///
    /// The following tasks are performed:
    ///
    ///   1. Persist the ClusterDiscovery to store.
    ///   2. Emit any discovery events if needed.
    ///   3. TODO: ensure cluster is in coordinator (zookeeper).
    ///   4. Pass the discovery to the status fetcher (TODO: move when coordinator is in place).
    ///   5. Pass the discovery to the status aggregator (TODO: move when coordinator is in place).
    fn process(&self, cluster: ClusterDiscovery) {
        let name = cluster.cluster.clone();
        if let Err(error) = self.process_checked(cluster) {
            let error = error.display_chain().to_string();
            error!(
                self.logger, "Failed to process cluster discovery";
                "cluster" => name, "error" => error
            );
            DISCOVERY_FETCH_ERRORS_COUNT.inc();
        }
    }

    fn process_checked(&self, cluster: ClusterDiscovery) -> Result<()> {
        self.process_discovery(cluster.clone())?;
        //self.ensure_coordination(cluster.clone())?;
        self.fetcher.process(cluster.clone());
        self.aggregator.process(cluster);
        Ok(())
    }

    fn process_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        match self.store.cluster_discovery(cluster.cluster.clone()) {
            Err(error) => Err(error).chain_err(|| FAIL_FIND_DISCOVERY),
            Ok(None) => self.process_discovery_new(cluster),
            Ok(Some(old)) => self.process_discovery_exising(cluster, old),
        }
    }

    fn process_discovery_exising(
        &self, cluster: ClusterDiscovery, old: ClusterDiscovery
    ) -> Result<()> {
        if cluster == old {
            return Ok(());
        }
        // TODO(stefano): emit cluster events based on new vs old.
        self.store.persist_discovery(cluster).chain_err(|| FAIL_PERSIST_DISCOVERY)
    }

    fn process_discovery_new(&self, cluster: ClusterDiscovery) -> Result<()> {
        let event = Event::builder().cluster().cluster_new(cluster.clone());
        self.store.persist_event(event).chain_err(|| FAIL_PERSIST_DISCOVERY)?;
        self.store.persist_discovery(cluster).chain_err(|| FAIL_PERSIST_DISCOVERY)
    }
}
