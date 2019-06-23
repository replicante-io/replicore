use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use slog::Logger;

use replicante_externals_kafka::AckLevel;
use replicante_externals_kafka::CommonConfig;
use replicante_externals_kafka::Timeouts;

use super::WorkerSet;
use crate::config::Backend;
use crate::config::KafkaConfig;
use crate::worker::backend::mock::MockBackend;
use crate::worker::backend::mock::TaskTemplate;
use crate::Config;
use crate::TaskQueue;

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
        let common = CommonConfig {
            ack_level: AckLevel::default(),
            brokers: "localhost:9092".into(),
            heartbeat: 3000,
            timeouts: Timeouts::default(),
        };
        let backend = Backend::Kafka(KafkaConfig {
            common,
            commit_retries: 8,
            queue_prefix: "queue".into(),
        });
        let config = Config {
            backend,
            threads_count: 2,
        };
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
