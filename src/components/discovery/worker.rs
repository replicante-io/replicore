use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::discover;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;
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
                    error!(self.logger, "Failed to fetch cluster discovery"; "error" => error);
                    DISCOVERY_FETCH_ERRORS_COUNT.inc();
                    continue;
                }
            };
            self.process(cluster);
        }
        debug!(self.logger, "Agents discovery complete");
    }
}

impl DiscoveryWorker {
    /// Persist an event to the store layer.
    ///
    /// Once the event stream layer is introduce also emit to that.
    fn emit_event(&self, event: Event) {
        if let Err(error) = self.store.persist_event(event) {
            let error = error.display_chain().to_string();
            error!(self.logger, "Failed to persist event"; "error" => error);
        }
    }

    /// Persist the discovery record to the store.
    fn persist_discovery(&self, cluster: ClusterDiscovery) {
        let name = cluster.name.clone();
        if let Err(error) = self.store.persist_discovery(cluster) {
            let error = error.display_chain().to_string();
            error!(
                self.logger, "Failed to persist cluster discovery";
                "cluster" => name, "error" => error
            );
        }
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
        self.process_discovery(cluster.clone());
        //self.ensure_coordination(cluster.clone());
        self.fetcher.process(cluster);
    }

    /// Process the discovery.
    ///
    /// The previous discovery result is used to determine changes.
    /// Events are emitted if there are any changes in the cluster.
    ///
    /// Once processing is complete the new cluster discovery is persisted.
    fn process_discovery(&self, cluster: ClusterDiscovery) {
        let old = match self.store.cluster_discovery(cluster.name.clone()) {
            Ok(old) => old,
            Err(error) => {
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to fetch cluster discovery";
                    "cluster" => cluster.name.clone(), "error" => error
                );
                return;
            }
        };

        // Inspect the previous discovery result (if any).
        match old {
            None => self.process_discovery_new(cluster),
            Some(old) => self.process_discovery_exising(cluster, old),
        };
    }

    /// Process an update to a discovery result and persist it.
    ///
    /// TODO: document emitted events.
    fn process_discovery_exising(&self, cluster: ClusterDiscovery, old: ClusterDiscovery) {
        // TODO: Emit cluster events based on new vs old.
        if cluster != old {
            self.persist_discovery(cluster);
        }
    }

    /// Process a new discovery result and persist it.
    ///
    /// Emit a ClusterNew event for the discovery.
    fn process_discovery_new(&self, cluster: ClusterDiscovery) {
        let event = Event::builder().cluster().new(cluster.clone());
        self.emit_event(event);
        self.persist_discovery(cluster);
    }
}
