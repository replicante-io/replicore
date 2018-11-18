use slog::Logger;

use replicante_data_models::ClusterDiscovery;
use replicante_tasks::TaskHandler;

use super::ReplicanteQueues;
use super::Result;
use super::Task;


/// Task handler for `ReplicanteQueues::Discovery` tasks.
pub struct Handler {
    logger: Logger,
}

impl Handler {
    pub fn new(logger: Logger) -> Handler {
        Handler { logger }
    }

    fn do_handle(&self, task: Task) -> Result<()> {
        let discovery: ClusterDiscovery = task.deserialize()?;
        debug!(
            self.logger, "TODO: implement discovery task";
            "discovery" => ?discovery, "task-id" => %task.id()
        );
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
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
