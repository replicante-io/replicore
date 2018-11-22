use std::sync::Arc;

use slog::Logger;


use super::Result;
use super::Task;
use super::TaskQueue;

use super::config::Backend as BackendConfig;
use super::config::Config;


mod backend;


use self::backend::kafka::Kafka;


/// Backend dependent admin logic.
trait AdminBackend<Q: TaskQueue> {
    /// See `TasksAdmin::scan` for details.
    fn scan(&self, queue: Q) -> Result<TasksIter<Q>>;
}


/// Additional task subsystem tools primarily for use by `replictl`.
pub struct TasksAdmin<Q: TaskQueue>(Arc<AdminBackend<Q>>);

impl<Q: TaskQueue> TasksAdmin<Q> {
    pub fn new(logger: Logger, config: Config) -> Result<TasksAdmin<Q>> {
        let backend = match config.backend.clone() {
            BackendConfig::Kafka(backend) => Arc::new(Kafka::new(logger.clone(), backend)?),
        };
        Ok(TasksAdmin(backend))
    }

    /// Iterate over all tasks (including skipped and to be retired tasks) on the given queue.
    pub fn scan(&self, queue: Q) -> Result<TasksIter<Q>> {
        self.0.scan(queue)
    }
}


///  Iterator over tasks stored in a queue.
pub struct TasksIter<Q: TaskQueue>(Box<Iterator<Item=Result<Task<Q>>>>);
impl<Q: TaskQueue> Iterator for TasksIter<Q> {
    type Item = Result<Task<Q>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
