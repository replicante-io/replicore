use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::discover;

use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;
use replicante_data_fetcher::Snapshotter;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;

use replicante_data_store::Store;
use replicante_streams_events::EventsStream;
use replicante_tasks::TaskRequest;

use super::super::super::Result;
use super::super::super::ResultExt;
use super::super::super::tasks::ReplicanteQueues;
use super::super::super::tasks::Tasks;
use super::EventsSnapshotsConfig;

use super::metrics::DISCOVERY_FETCH_ERRORS_COUNT;
use super::metrics::DISCOVERY_SNAPSHOT_TRACKER_COUNT;


const FAIL_FIND_DISCOVERY: &str = "Failed to fetch cluster discovery";
const FAIL_PERSIST_DISCOVERY: &str = "Failed to persist cluster discovery";


/// Implements the discovery logic of a signle discovery loop.
pub struct DiscoveryWorker {
    discovery_config: BackendsConfig,
    emissions: EmissionTracker,
    events: EventsStream,
    logger: Logger,
    store: Store,
    tasks: Tasks,

    // TODO(stefano): move into dedicated component when possible.
    aggregator: Aggregator,
    fetcher: Fetcher,
}

impl DiscoveryWorker {
    /// Creates a discover worker.
    pub fn new(
        discovery_config: BackendsConfig, snapshots_config: EventsSnapshotsConfig,
        logger: Logger, events: EventsStream, store: Store, tasks: Tasks, timeout: Duration
    ) -> DiscoveryWorker {
        let aggregator = Aggregator::new(logger.clone(), store.clone());
        let fetcher = Fetcher::new(logger.clone(), events.clone(), store.clone(), timeout);
        let emissions = EmissionTracker::new(snapshots_config);
        DiscoveryWorker {
            discovery_config,
            emissions,
            events,
            logger,
            store,
            tasks,

            // TODO(stefano): move into dedicated component when possible.
            aggregator,
            fetcher,
        }
    }

    /// Runs a signle discovery loop.
    pub fn run(&self) {
        debug!(self.logger, "Discovering agents ...");
        for cluster in discover(self.discovery_config.clone()) {
            match cluster {
                Ok(cluster) => {
                    self.process(cluster.clone());
                    let task = TaskRequest::new(ReplicanteQueues::ClusterRefresh);
                    if let Err(error) = self.tasks.request(task, cluster) {
                        error!(self.logger, "Failed to request cluster discovery"; "error" => %error);
                        DISCOVERY_FETCH_ERRORS_COUNT.inc();
                    };
                },
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
    ///   3. TODO: Emit cluster refresh task.
    ///   4. Pass the discovery to the status fetcher (TODO: move when tasks are in place).
    ///   5. Pass the discovery to the status aggregator (TODO: move when tasks are in place).
    fn process(&self, cluster: ClusterDiscovery) {
        let name = cluster.cluster.clone();
        let snapshot = self.emissions.snapshot(name.clone());
        if let Err(error) = self.process_checked(cluster, snapshot) {
            let error = error.display_chain().to_string();
            error!(
                self.logger, "Failed to process cluster discovery";
                "cluster" => name, "error" => error
            );
            DISCOVERY_FETCH_ERRORS_COUNT.inc();
        }
    }

    fn process_checked(&self, cluster: ClusterDiscovery, snapshot: bool) -> Result<()> {
        let name = cluster.cluster.clone();
        if snapshot {
            let snapshotter = Snapshotter::new(
                name.clone(), self.events.clone(), self.store.clone()
            );
            if let Err(error) = snapshotter.run() {
                let error = error.display_chain().to_string();
                error!(
                    self.logger, "Failed to emit snapshots";
                    "cluster" => name, "error" => error
                );
            }
        }
        self.process_discovery(cluster.clone())?;
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
        let event = Event::builder().cluster().changed(old, cluster.clone());
        self.events.emit(event).chain_err(|| FAIL_PERSIST_DISCOVERY)?;
        self.store.persist_discovery(cluster).chain_err(|| FAIL_PERSIST_DISCOVERY)
    }

    fn process_discovery_new(&self, cluster: ClusterDiscovery) -> Result<()> {
        let event = Event::builder().cluster().cluster_new(cluster.clone());
        self.events.emit(event).chain_err(|| FAIL_PERSIST_DISCOVERY)?;
        self.store.persist_discovery(cluster).chain_err(|| FAIL_PERSIST_DISCOVERY)
    }
}


/// Helper object to decide when snapshot events should be emitted.
///
/// This is a naive implementation using an in-memory map that never forgets nodes.
/// As a consequence it can "leak" memory in the presence of frequent cluster rotation.
struct EmissionTracker {
    enabled: bool,
    frequency: u32,
    state: Mutex<HashMap<String, u32>>,
}

impl EmissionTracker {
    pub fn new(config: EventsSnapshotsConfig) -> EmissionTracker {
        EmissionTracker {
            enabled: config.enabled,
            frequency: config.frequency,
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Determine if it is time to snapshot a cluster.
    pub fn snapshot(&self, cluster: String) -> bool {
        if !self.enabled {
            return false;
        }
        let mut map = self.state.lock().expect("EmissionTracker lock was poisoned");
        // Default to 1 so that we can emit immediatelly.
        // This is so that a failover leads to a double snapshot instead of a snapshot delay.
        let state = map.entry(cluster).or_insert_with(|| {
            DISCOVERY_SNAPSHOT_TRACKER_COUNT.inc();
            1
        });
        *state -= 1;
        if *state == 0 {
            *state = self.frequency;
            return true;
        }
        false
    }
}
