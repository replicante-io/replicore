use failure::Fail;
use failure::ResultExt;
use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_data_models::ClusterDiscovery;
use replicante_tasks::TaskHandler;
// TODO(stefano): once error_chain is gone use replicante_util_failure::failure_info;

use super::super::super::ErrorKind;
use super::super::super::Result;
use super::Interfaces;
use super::ReplicanteQueues;
use super::Task;


/// Task handler for `ReplicanteQueues::Discovery` tasks.
pub struct Handler {
    coordinator: Coordinator,
    logger: Logger,
}

impl Handler {
    pub fn new(interfaces: &Interfaces, logger: Logger) -> Handler {
        let coordinator = interfaces.coordinator.clone();
        Handler {
            coordinator,
            logger,
        }
    }

    fn do_handle(&self, task: &Task) -> Result<()> {
        let discovery: ClusterDiscovery = task.deserialize()?;

        // Ensure only one refresh at the same time.
        let mut lock = self.coordinator.non_blocking_lock(
            format!("cluster_refresh/{}", discovery.cluster)
        );
        match lock.acquire() {
            Ok(()) => (),
            Err(error) => {
                match error.kind() {
                    ::replicante_coordinator::ErrorKind::LockHeld(_, owner) => {
                        // TODO: increment per-cluster counter of locked clusters.
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
        debug!(
            self.logger, "TODO: implement discovery task";
            "discovery" => ?discovery, "task-id" => %task.id()
        );
        ::std::thread::sleep(::std::time::Duration::from_secs(10));

        // Done.
        lock.release().context(ErrorKind::Coordination)?;
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
