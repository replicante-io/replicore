use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use slog::Logger;

use super::super::super::Config;
use super::super::backend::mock::MockBackend;
use super::super::backend::mock::TaskTemplate;
use super::super::TaskQueue;
use super::WorkerSet;

/// Mock tools to test `WorkerSet` users.
#[derive(Default)]
pub struct MockWorkerSet<Q: TaskQueue> {
    pub tasks: Arc<Mutex<VecDeque<TaskTemplate<Q>>>>,
}

impl<Q: TaskQueue> MockWorkerSet<Q> {
    /// Create a mock tasks instance to be used for tests.
    pub fn new() -> MockWorkerSet<Q> {
        MockWorkerSet {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Return the non-mock interface to interact with this mock using the default configuration.
    pub fn mock(&self, logger: Logger) -> WorkerSet<Q> {
        let mut config = Config::default();
        config.threads_count = 2;
        self.mock_with_config(logger, config)
    }

    /// Return the non-mock interface to interact with this mock.
    pub fn mock_with_config(&self, logger: Logger, config: Config) -> WorkerSet<Q> {
        let backend = Arc::new(MockBackend {
            tasks: self.tasks.clone(),
        });
        WorkerSet {
            backend,
            config,
            handlers: HashMap::new(),
            logger,
        }
    }
}
