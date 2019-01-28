use std::time::Duration;

use failure::Fail;
use failure::ResultExt;
use failure::err_msg;
use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_data_aggregator::Aggregator;
use replicante_data_fetcher::Fetcher;
use replicante_data_fetcher::Snapshotter;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::Event;
use replicante_data_store::Store;
use replicante_streams_events::EventsStream;
use replicante_tasks::TaskHandler;
// TODO(stefano): once error_chain is gone use replicante_util_failure::failure_info;

use super::super::super::Error;
use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::super::task_payload::ClusterRefreshPayload;
use super::Interfaces;
use super::ReplicanteQueues;
use super::Task;


mod metrics;

pub use self::metrics::register_metrics;
use self::metrics::REFRESH_DURATION;
use self::metrics::REFRESH_LOCKED;


const FAIL_PERSIST_DISCOVERY: &str = "Failed to persist cluster discovery";


/// Task handler for `ReplicanteQueues::Discovery` tasks.
pub struct Handler {
    aggregator: Aggregator,
    coordinator: Coordinator,
    events: EventsStream,
    fetcher: Fetcher,
    logger: Logger,
    store: Store,
}

impl Handler {
    pub fn new(interfaces: &Interfaces, logger: Logger, agents_timeout: Duration) -> Handler {
        let aggregator = Aggregator::new(logger.clone(), interfaces.store.clone());
        let coordinator = interfaces.coordinator.clone();
        let events = interfaces.streams.events.clone();
        let fetcher = Fetcher::new(
            logger.clone(), interfaces.streams.events.clone(), interfaces.store.clone(),
            agents_timeout
        );
        let store = interfaces.store.clone();
        Handler {
            aggregator,
            coordinator,
            events,
            fetcher,
            logger,
            store,
        }
    }

    fn do_handle(&self, task: &Task) -> Result<()> {
        let payload: ClusterRefreshPayload = task.deserialize()?;
        let discovery = payload.cluster;
        let snapshot = payload.snapshot;

        // Ensure only one refresh at the same time.
        let mut lock = self.coordinator.non_blocking_lock(
            format!("cluster_refresh/{}", discovery.cluster)
        );
        match lock.acquire() {
            Ok(()) => (),
            Err(error) => {
                match error.kind() {
                    ::replicante_coordinator::ErrorKind::LockHeld(_, owner) => {
                        REFRESH_LOCKED.with_label_values(&[&discovery.cluster]).inc();
                        info!(
                            self.logger,
                            "Skipped cluster refresh because another task is in progress";
                            "cluster" => discovery.cluster, "owner" => %owner
                        );
                        return Ok(());
                    },
                    _ => (),
                };
                return Err(error.context(ErrorKind::Coordination).into());
            }
        };

        // Refresh cluster state.
        let timer = REFRESH_DURATION.with_label_values(&[&discovery.cluster]).start_timer();
        self.emit_snapshots(&discovery.cluster, snapshot)?;
        self.refresh_discovery(discovery.clone())?;
        self.fetcher.process(discovery.clone(), lock.watch());
        self.aggregator.process(discovery, lock.watch());

        // Done.
        timer.observe_duration();
        lock.release().context(ErrorKind::Coordination)?;
        Ok(())
    }

    /// Emit cluster state snapshots, if needed by this task.
    fn emit_snapshots(&self, name: &str, snapshot: bool) -> Result<()> {
        if !snapshot {
            return Ok(());
        }
        debug!(self.logger, "Emitting cluster snapshot"; "cluster" => name);
        let snapshotter = Snapshotter::new(name.into(), self.events.clone(), self.store.clone());
        if let Err(error) = snapshotter.run() {
            error!(
                self.logger, "Failed to emit snapshots";
                "cluster" => name, "error" => %error
                // TODO: failure_info(&error)
            );
        }
        Ok(())
    }

    /// Refresh the state of the cluster discovery.
    ///
    /// Refresh is performed based on the current state or luck of state.
    /// This method emits events as needed (and before the state is updated).
    fn refresh_discovery(&self, discovery: ClusterDiscovery) -> Result<()> {
        let current_state = self.store.cluster_discovery(discovery.cluster.clone())?;
        if let Some(current_state) = current_state {
            if discovery == current_state {
                return Ok(());
            }
            let event = Event::builder().cluster().changed(current_state, discovery.clone());
            self.events.emit(event).map_err(Error::from)
                .context(ErrorKind::Legacy(err_msg(FAIL_PERSIST_DISCOVERY)))
                .map_err(Error::from)?;
        } else {
            let event = Event::builder().cluster().cluster_new(discovery.clone());
            self.events.emit(event).map_err(Error::from)
                .context(ErrorKind::Legacy(err_msg(FAIL_PERSIST_DISCOVERY)))
                .map_err(Error::from)?;
        }
        self.store.persist_discovery(discovery).map_err(Error::from)
            .context(ErrorKind::Legacy(err_msg(FAIL_PERSIST_DISCOVERY)))
            .map_err(Error::from)?;
        Ok(())
    }
}

impl TaskHandler<ReplicanteQueues> for Handler {
    fn handle(&self, task: Task) {
        match self.do_handle(&task) {
            Ok(()) => {
                if let Err(error) = task.success() {
                    error!(
                        self.logger, "Error while acking successfully processed task";
                        "error" => ?error
                        // TODO(stefano): once error_chain is gone: failure_info(&error)
                    );
                }
            },
            Err(error) => {
                error!(
                    self.logger, "Failed to handle cluster discovery task";
                    "error" => ?error
                    // TODO(stefano): once error_chain is gone: failure_info(&error)
                );
                if let Err(error) = task.fail() {
                    error!(
                        self.logger, "Error while acking failed task";
                        "error" => ?error
                        // TODO(stefano): once error_chain is gone: failure_info(&error)
                    );
                }
            }
        }
    }
}
