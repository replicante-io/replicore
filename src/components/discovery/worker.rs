use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::discover;

use replicante_data_models::ClusterDiscovery;
use replicante_data_store::Store;

use super::metrics::DISCOVERY_FETCH_ERRORS_COUNT;
use super::statefetch::Fetcher;


/// Implements the discovery logic of a signle discovery loop.
pub struct DiscoveryWorker {
    config: BackendsConfig,
    logger: Logger,
    store: Store,

    // TODO: move into dedicated component when possible.
    fetcher: Fetcher,
}

impl DiscoveryWorker {
    /// Creates a discover worker.
    pub fn new(config: BackendsConfig, logger: Logger, store: Store) -> DiscoveryWorker {
        let fetcher = Fetcher::new(logger.clone(), store.clone());
        DiscoveryWorker {
            config,
            logger,
            store,

            // TODO: move into dedicated component when possible.
            fetcher,
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

    /// Persist and process the discovery.
    ///
    /// Once the discovery is persited the previous result is used to determine changes.
    /// Events are emitted if there are any changes in the cluster.
    fn persist_discovery(&self, cluster: ClusterDiscovery) {
        let _old = match self.store.persist_discovery(cluster.clone()) {
            Ok(old) => old,
            Err(error) => {
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to persist cluster discovery";
                    "cluster" => cluster.name.clone(), "error" => error
                );
                return;
            }
        };
        // TODO: figure out if the cluster changed.
    }

    /// Process a discovery result to fetch the node state.
    ///
    /// The following tasks are performed:
    ///
    ///   1. Persist the ClusterDiscovery to store.
    ///   2. Emit any discovery events if needed.
    ///   3. TODO: ensure cluster is in coordinator (zookeeper, when datafetch is split).
    ///   4. Pass the discovery to the state fetcher (TODO: move after coordinator is in place).
    fn process(&self, cluster: ClusterDiscovery) {
        self.persist_discovery(cluster.clone());
        //self.ensure_coordination(cluster);
        self.fetcher.process(cluster);
    }
}
