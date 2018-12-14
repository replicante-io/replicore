use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_data_models::ClusterDiscovery;
use replicante_tasks::TaskHandler;

//use super::super::super::ErrorKind;
use super::super::super::Result;
use super::Interfaces;
use super::ReplicanteQueues;
use super::Task;


/// Task handler for `ReplicanteQueues::Discovery` tasks.
pub struct Handler {
    _coordinator: Coordinator,
    logger: Logger,
}

impl Handler {
    pub fn new(interfaces: &Interfaces, logger: Logger) -> Handler {
        let coordinator = interfaces.coordinator.clone();
        Handler {
            _coordinator: coordinator,
            logger,
        }
    }

    fn do_handle(&self, task: Task) -> Result<()> {
        let discovery: ClusterDiscovery = task.deserialize()?;

        // TODO(stefano): Skip processing if a tombstone exists for the cluster.
        //let tombstone = self.coordinator.tombstone(format!("discovery/{}", discovery.cluster));
        //match tombstone.check().context(ErrorKind::Coordination) {
        //    Err(err) => {
        //        task.fail()?;
        //        return Err(err)?;
        //    },
        //    Ok(Some(node)) => {
        //        info!(
        //            self.logger, "Cluster refresh task throttled";
        //            "cluster" => discovery.cluster, "by_node" => %node,
        //            "this_node" => %self.coordinator.node_id()
        //        );
        //        task.success()?;
        //        return Ok(());
        //    },
        //    _ => (),
        //};

        // Refresh cluster state.
        debug!(
            self.logger, "TODO: implement discovery task";
            "discovery" => ?discovery, "task-id" => %task.id()
        );
        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        // Create tombstone and ack task.
        // TODO: Create tombstone.
        task.success()?;
        Ok(())
    }
}

impl TaskHandler<ReplicanteQueues> for Handler {
    fn handle(&self, task: Task) {
        if let Err(error) = self.do_handle(task) {
            error!(self.logger, "Failed to handle cluster discovery task"; "error" => ?error);
        }
    }
}
