use std::sync::Arc;
use std::time::Duration;

use slog::Logger;

use replicante_service_coordinator::mock::MockCoordinator;
use replicante_store_primary::mock::Mock as MockPrimaryStore;
use replicante_store_view::mock::Mock as MockViewStore;
use replicante_stream_events::Stream as EventsStream;

use super::HealthChecks;
use super::Interfaces;
use super::Metrics;
use super::Stores;
use super::Streams;
use super::Tracing;
use super::API;
use crate::tasks::MockTasks;

/// A container for mocks used by interfaces.
#[cfg(test)]
pub struct MockInterfaces {
    pub coordinator: MockCoordinator,
    pub logger: Logger,
    pub stores: MockStores,
    pub tasks: Arc<MockTasks>,
}

#[cfg(test)]
impl MockInterfaces {
    /// Mock interfaces and wrap them in an `Interfaces` instance.
    ///
    /// This method will use a JSON logger to stdout.
    /// Use `Interfaces::mock_with_logger` to specify the logger.
    pub fn mock() -> MockInterfaces {
        let logger_opts = replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
        let logger = replicante_logging::starter(&logger_opts);
        MockInterfaces::mock_with_logger(logger)
    }

    /// Mock interfaces discarding logs.
    pub fn mock_quietly() -> MockInterfaces {
        let logger = Logger::root(slog::Discard, slog::o!());
        MockInterfaces::mock_with_logger(logger)
    }

    /// Mock interfaces using the given logger and wrap them in an `Interfaces` instance.
    pub fn mock_with_logger(logger: Logger) -> MockInterfaces {
        let coordinator = MockCoordinator::new(logger.clone());
        let stores = MockStores::new();
        let tasks = Arc::new(MockTasks::new());
        MockInterfaces {
            coordinator,
            logger,
            stores,
            tasks,
        }
    }

    pub fn interfaces(&self) -> Interfaces {
        let metrics = Metrics::mock();
        let healthchecks = HealthChecks::new(Duration::from_secs(10));
        let tracing = Tracing::mock();
        let (api, _) = API::mock(self.logger.clone(), &metrics, healthchecks.results_proxy());
        let coordinator = self.coordinator.mock();
        let events = EventsStream::mock();
        let stores = self.stores.mock();
        Interfaces {
            api,
            coordinator,
            healthchecks,
            logger: self.logger.clone(),
            metrics,
            stores,
            streams: Streams { events },
            tasks: self.tasks.mock(),
            tracing,
        }
    }
}

/// A container for stores mocks.
#[cfg(test)]
pub struct MockStores {
    pub primary: MockPrimaryStore,
    pub view: MockViewStore,
}

impl MockStores {
    pub fn new() -> MockStores {
        MockStores {
            primary: MockPrimaryStore::default(),
            view: MockViewStore::default(),
        }
    }

    pub fn mock(&self) -> Stores {
        Stores {
            primary: self.primary.store(),
            view: self.view.store(),
        }
    }
}
