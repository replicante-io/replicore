use failure::ResultExt;
use prometheus::Registry;
use slog::Logger;

use replicante_coordinator::Coordinator;
use replicante_data_store::store::Store;
use replicante_streams_events::EventsStream;

use super::ErrorKind;
use super::Result;
use super::config::Config;
use super::tasks::Tasks;


pub mod api;
pub mod metrics;
pub mod tracing;

use self::api::API;
use self::metrics::Metrics;
use self::tracing::Tracing;


/// A container for replicante interfaces.
///
/// This container is useful to:
///
///   1. Have one argument passed arround for injection instead of many.
///   2. Store thread [`JoinHandle`]s to join on [`Drop`].
///
/// [`Drop`]: std/ops/trait.Drop.html
/// [`JoinHandle`]: std/thread/struct.JoinHandle.html
pub struct Interfaces {
    pub api: API,
    pub coordinator: Coordinator,
    pub metrics: Metrics,
    pub store: Store,
    pub streams: Streams,
    pub tasks: Tasks,
    pub tracing: Tracing,
}

impl Interfaces {
    /// Creates and configures interfaces.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: &Config, logger: Logger) -> Result<Interfaces> {
        let metrics = Metrics::new();
        let coordinator = Coordinator::new(config.coordinator.clone(), logger.clone())
            .with_context(|_| ErrorKind::InterfaceInit("coordinator"))?;
        let api = API::new(config.api.clone(), coordinator.clone(), logger.clone(), &metrics);
        let store = Store::make(config.storage.clone(), logger.clone())
            .with_context(|_| ErrorKind::ClientInit("store"))?;
        let streams = Streams::new(config, logger.clone(), store.clone())?;
        let tasks = Tasks::new(config.tasks.clone())
            .with_context(|_| ErrorKind::ClientInit("tasks"))?;
        let tracing = Tracing::new(config.tracing.clone(), logger.clone())?;
        Ok(Interfaces {
            api,
            coordinator,
            metrics,
            store,
            streams,
            tasks,
            tracing,
        })
    }

    /// Attemps to register all interfaces metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        ::replicante_coordinator::register_metrics(logger, registry);
        ::replicante_data_store::register_metrics(logger, registry);
        ::replicante_tasks::register_metrics(logger, registry);
        EventsStream::register_metrics(logger, registry);
        self::api::register_metrics(logger, registry);
        self::metrics::register_metrics(logger, registry);
    }

    /// Performs any final configuration and starts background threads.
    ///
    /// For example, the [`ApiInterface`] uses it to wrap the router into a server.
    pub fn run(&mut self) -> Result<()> {
        self.api.run()?;
        self.metrics.run()?;
        self.tracing.run()?;
        Ok(())
    }

    /// Waits for all interfaces to terminate.
    pub fn wait_all(&mut self) -> Result<()> {
        self.api.wait()?;
        self.metrics.wait()?;
        self.tracing.wait()?;
        Ok(())
    }
}


/// Collection of all the streaming interfaces.
pub struct Streams {
    pub events: EventsStream,
}

impl Streams {
    pub fn new(config: &Config, logger: Logger, store: Store) -> Result<Streams> {
        let events = EventsStream::new(config.events.stream.clone(), logger, store);
        Ok(Streams {
            events,
        })
    }
}


// *** Implement interfaces mocks for tests *** //
/// A container for mocks used by interfaces.
#[cfg(test)]
pub struct MockInterfaces {
    pub coordinator: ::replicante_coordinator::mock::MockCoordinator,
    pub events: ::std::sync::Arc<::replicante_streams_events::mock::MockEvents>,
    pub store: ::replicante_data_store::mock::Mock,
    pub tasks: ::std::sync::Arc<super::tasks::MockTasks>,
}

#[cfg(test)]
impl Interfaces {
    /// Mock interfaces and wrap them in an `Interfaces` instance.
    ///
    /// This method will use a JSON logger to stdout.
    /// Use `Interfaces::mock_with_logger` to specify the logger.
    pub fn mock() -> (Interfaces, MockInterfaces) {
        let logger_opts = ::replicante_logging::Opts::new(env!("GIT_BUILD_HASH").into());
        let logger = ::replicante_logging::starter(&logger_opts);
        Interfaces::mock_with_logger(logger)
    }

    /// Mock interfaces using the given logger and wrap them in an `Interfaces` instance.
    pub fn mock_with_logger(logger: Logger) -> (Interfaces, MockInterfaces) {
        let metrics = Metrics::mock();
        let (api, _) = API::mock(logger.clone(), &metrics);
        let tracing = Tracing::mock();

        let mock_coordinator = ::replicante_coordinator::mock::MockCoordinator::new(logger.clone());
        let coordinator = mock_coordinator.mock();

        let mock_events = ::replicante_streams_events::mock::MockEvents::new();
        let mock_events = ::std::sync::Arc::new(mock_events);
        let events = ::replicante_streams_events::mock::MockEvents::mock(mock_events.clone());

        let mock_store = ::replicante_data_store::mock::Mock::default();
        let store = mock_store.store();
        let tasks = ::std::sync::Arc::new(super::tasks::MockTasks::new());

        // Wrap things up.
        let mocks = MockInterfaces {
            coordinator: mock_coordinator,
            events: mock_events,
            store: mock_store,
            tasks,
        };
        let interfaces = Interfaces {
            api,
            coordinator,
            metrics,
            store,
            streams: Streams {
                events,
            },
            tasks: mocks.tasks.mock(),
            tracing,
        };
        (interfaces, mocks)
    }
}


#[cfg(test)]
mod tests {
    use super::Interfaces;

    #[test]
    fn instantiate_mocks() {
        let (interfaces, mocks) = Interfaces::mock();
        drop(interfaces);
        drop(mocks);
    }
}
