use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use opentracingrust::Tracer;
use prometheus::Registry;
use slog::Logger;

use replicante_service_coordinator::Coordinator;
use replicante_service_healthcheck::HealthChecks as HealthChecksRegister;
use replicante_store_primary::store::Store as PrimaryStore;
use replicante_store_view::store::Store as ViewStore;
use replicante_stream_events::Stream as EventsStream;
use replicante_util_upkeep::Upkeep;

use replicore_models_tasks::Tasks;

use super::config::Config;
use super::ErrorKind;
use super::Result;

pub mod api;
mod healthchecks;
pub mod metrics;
pub mod tracing;

#[cfg(test)]
pub mod test_support;

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
    pub logger: Logger,
    pub metrics: Metrics,
    pub stores: Stores,
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
            config.clone(),
            coordinator.clone(),
            logger.clone(),
            &metrics,
            healthchecks.results_proxy(),
        );
        let stores = Stores::new(
            &config,
            logger.clone(),
            healthchecks.register(),
            tracing.tracer(),
        )?;
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
            logger,
            metrics,
            stores,
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

/// Collection of all the storage interfaces.
pub struct Stores {
    pub primary: PrimaryStore,
    pub view: ViewStore,
}

impl Stores {
    pub fn new(
        config: &Config,
        logger: Logger,
        healthchecks: &mut HealthChecksRegister,
        tracer: Arc<Tracer>,
    ) -> Result<Stores> {
        let primary = PrimaryStore::make(
            config.storage.primary.clone(),
            logger.clone(),
            healthchecks,
            Arc::clone(&tracer),
        )
        .with_context(|_| ErrorKind::ClientInit("primary store"))?;
        let view = ViewStore::new(config.storage.view.clone(), logger, healthchecks, tracer)
            .with_context(|_| ErrorKind::ClientInit("view store"))?;
        Ok(Stores { primary, view })
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

#[cfg(test)]
mod tests {
    use super::test_support::MockInterfaces;

    #[test]
    fn instantiate_mocks() {
        let mocks = MockInterfaces::mock();
        let _interfaces = mocks.interfaces();
    }
}
