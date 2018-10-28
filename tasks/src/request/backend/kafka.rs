//use super::super::super::TaskError;
use super::super::super::config::KafkaConfig;

use super::Backend;
use super::Result;
use super::TaskQueue;
use super::TaskRequest;


/// Kafka-backed tasks queue system.
///
/// TODO: document queues (base, _retry, _trash)
/// TODO: document retry system
pub struct Kafka {
    // TODO
}

impl Kafka {
    pub fn new(_config: KafkaConfig) -> Kafka {
        Kafka {
            // TODO
        }
    }
}

impl<Q: TaskQueue> Backend<Q> for Kafka {
    fn request(&self, _task: TaskRequest<Q>, _message: &[u8]) -> Result<()> {
        // TODO: Implement kafka
        Ok(())
    }
}
