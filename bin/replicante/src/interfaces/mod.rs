use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::Tracer;
use prometheus::Registry;
use slog::Logger;

use replicante_service_coordinator::Coordinator;
use replicante_service_healthcheck::HealthChecks as HealthChecksRegister;
use replicante_store_primary::store::Store;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_upkeep::Upkeep;

use super::config::Config;
use super::tasks::Tasks;
use super::ErrorKind;
use super::Result;

pub mod api;
mod healthchecks;
pub mod metrics;
pub mod tracing;

use self::api::API;
pub use self::healthchecks::HealthChecks;
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
    pub healthchecks: HealthChecks,
    pub metrics: Metrics,
    pub store: Store,
    pub streams: Streams,
    pub tasks: Tasks,
    pub tracing: Tracing,
}

impl Interfaces {
    /// Creates and configures interfaces.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: &Config, logger: Logger, upkeep: &mut Upkeep) -> Result<Interfaces> {
        let metrics = Metrics::new();
        let mut healthchecks =
            HealthChecks::new(Duration::from_secs(config.api.healthcheck_refresh));
        let tracing = Tracing::new(config.tracing.clone(), logger.clone(), upkeep)?;
        let coordinator = Coordinator::new(
            config.coordinator.clone(),
            logger.clone(),
            healthchecks.register(),
            tracing.tracer(),
        )
        .with_context(|_| ErrorKind::InterfaceInit("coordinator"))?;
        let api = API::new(
            config.api.clone(),
            config
                .sentry
                .as_ref()
                .map(|sentry| sentry.capture_api_errors.clone())
                .unwrap_or_default(),
            coordinator.clone(),
            logger.clone(),
            &metrics,
            healthchecks.results_proxy(),
            tracing.tracer(),
        );
        let store = Store::make(
            config.storage.clone(),
            logger.clone(),
            healthchecks.register(),
            tracing.tracer(),
        )
        .with_context(|_| ErrorKind::ClientInit("store"))?;
        let streams = Streams::new(
            config,
            logger.clone(),
            healthchecks.register(),
            tracing.tracer(),
        )?;
        let tasks = Tasks::new(config.tasks.clone(), healthchecks.register())
            .with_context(|_| ErrorKind::ClientInit("tasks"))?;
        Ok(Interfaces {
            api,
            coordinator,
            healthchecks,
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
        self::api::register_metrics(logger, registry);
        self::metrics::register_metrics(logger, registry);
    }

    /// Performs any final configuration and starts background threads.
    ///
    /// For example, the [`ApiInterface`] uses it to wrap the router into a server.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        self.api.run(upkeep)?;
        self.healthchecks.run(upkeep)?;
        self.metrics.run()?;
        self.tracing.run()?;
        Ok(())
    }
}

/// Collection of all the streaming interfaces.
pub struct Streams {
    pub events: EventsStream,
}

impl Streams {
    pub fn new(
        config: &Config,
        logger: Logger,
        healthchecks: &mut HealthChecksRegister,
        tracer: Arc<Tracer>,
    ) -> Result<Streams> {
        let events = EventsStream::new(config.events.stream.clone(), logger, healthchecks, tracer)
            .with_context(|_| ErrorKind::InterfaceInit("events stream"))?;
        Ok(Streams { events })
    }
}

// *** Implement interfaces mocks for tests *** //
/// A container for mocks used by interfaces.
#[cfg(test)]
pub struct MockInterfaces {
    pub coordinator: replicante_service_coordinator::mock::MockCoordinator,
    pub store: replicante_store_primary::mock::Mock,
    pub tasks: std::sync::Arc<super::tasks::MockTasks>,
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
        let healthchecks = HealthChecks::new(Duration::from_secs(10));
        let tracing = Tracing::mock();
        let (api, _) = API::mock(
            logger.clone(),
            &metrics,
            healthchecks.results_proxy(),
            tracing.tracer(),
        );

        let mock_coordinator =
            replicante_service_coordinator::mock::MockCoordinator::new(logger.clone());
        let coordinator = mock_coordinator.mock();
        let events = replicante_stream_events::Stream::mock();

        let mock_store = replicante_store_primary::mock::Mock::default();
        let store = mock_store.store();
        let tasks = std::sync::Arc::new(super::tasks::MockTasks::new());

        // Wrap things up.
        let mocks = MockInterfaces {
            coordinator: mock_coordinator,
            store: mock_store,
            tasks,
        };
        let interfaces = Interfaces {
            api,
            coordinator,
            healthchecks,
            metrics,
            store,
            streams: Streams { events },
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
