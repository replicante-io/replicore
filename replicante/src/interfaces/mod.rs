use prometheus::Registry;
use slog::Logger;

use replicante_data_store::Store;
use replicante_streams_events::EventsStream;

use super::Result;
use super::config::Config;


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
    pub metrics: Metrics,
    pub store: Store,
    pub streams: Streams,
    pub tracing: Tracing,
}

impl Interfaces {
    /// Creates and configures interfaces.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn new(config: &Config, logger: Logger) -> Result<Interfaces> {
        let metrics = Metrics::new();
        let api = API::new(config.api.clone(), logger.clone(), &metrics);
        let store = Store::new(config.storage.clone(), logger.clone())?;
        let streams = Streams::new(config, logger.clone(), store.clone())?;
        let tracing = Tracing::new(config.tracing.clone(), logger.clone())?;
        Ok(Interfaces {
            api,
            metrics,
            store,
            streams,
            tracing,
        })
    }

    /// Attemps to register all interfaces metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        self::api::register_metrics(logger, registry);
        self::metrics::register_metrics(logger, registry);
        EventsStream::register_metrics(logger, registry);
        Store::register_metrics(logger, registry);
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
    pub events: ::std::sync::Arc<::replicante_streams_events::mock::MockEvents>,
    pub store: ::std::sync::Arc<::replicante_data_store::mock::MockStore>,
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
        let api = API::mock(logger.clone(), &metrics);
        let tracing = Tracing::mock();

        let mock_events = ::replicante_streams_events::mock::MockEvents::new();
        let mock_events = ::std::sync::Arc::new(mock_events);
        let events = ::replicante_streams_events::mock::MockEvents::mock(mock_events.clone());

        let mock_store = ::replicante_data_store::mock::MockStore::new();
        let mock_store = ::std::sync::Arc::new(mock_store);
        let store = Store::mock(mock_store.clone());

        // Wrap things up.
        let mocks = MockInterfaces {
            events: mock_events,
            store: mock_store,
        };
        let interfaces = Interfaces {
            api,
            metrics,
            store,
            streams: Streams {
                events,
            },
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
