use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_yaml;

use replicante_logging::Config as LoggingConfig;
use replicante_logging::LoggingLevel;
use replicante_service_coordinator::Config as CoordinatorConfig;
use replicante_service_tasks::Config as TasksConfig;
use replicante_store_primary::Config as StorageConfig;
use replicante_util_tracing::Config as TracingConfig;

use super::components::DiscoveryConfig;
use super::interfaces::api::Config as APIConfig;
use super::ErrorKind;
use super::Result;

mod components;
mod events;
mod sentry;
mod task_workers;
mod timeouts;

pub use self::components::ComponentsConfig;
pub use self::events::EventsConfig;
pub use self::events::SnapshotsConfig as EventsSnapshotsConfig;
pub use self::sentry::SentryCaptureApi;
pub use self::sentry::SentryConfig;
pub use self::task_workers::TaskWorkers;
pub use self::timeouts::TimeoutsConfig;

/// Replicante configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// API server configuration.
    #[serde(default)]
    pub api: APIConfig,

    /// Components enabling configuration.
    #[serde(default)]
    pub components: ComponentsConfig,

    /// Distributed coordinator configuration options.
    #[serde(default)]
    pub coordinator: CoordinatorConfig,

    /// Agent discovery configuration.
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Events configuration.
    pub events: EventsConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Optional sentry integration configuration.
    #[serde(default)]
    pub sentry: Option<SentryConfig>,

    /// Storage layer configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Task workers enabling configuration.
    #[serde(default)]
    pub task_workers: TaskWorkers,

    /// Tasks system configuration.
    pub tasks: TasksConfig,

    /// Timeouts configured here are used throughout the system for various reasons.
    #[serde(default)]
    pub timeouts: TimeoutsConfig,

    /// Distributed tracing configuration.
    #[serde(default)]
    pub tracing: TracingConfig,
}

impl Config {
    /// Loads the configuration from the given [`std::fs::File`].
    ///
    /// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let config = File::open(path).with_context(|_| ErrorKind::ConfigLoad)?;
        Config::from_reader(config)
    }

    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn from_reader<R: Read>(reader: R) -> Result<Config> {
        let conf = serde_yaml::from_reader(reader).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
    }

    /// Apply transformations to the configuration to derive some parameters.
    ///
    /// Transvormation:
    ///
    ///   * Apply verbose debug level logic.
    pub fn transform(mut self) -> Self {
        // With !verbose logging debug level applies only to replicante crates.
        if self.logging.level == LoggingLevel::Debug && !self.logging.verbose {
            self.logging.level = LoggingLevel::Info;
            self.logging
                .modules
                .entry("replicante".into())
                .or_insert(LoggingLevel::Debug);
            self.logging
                .modules
                .entry("replictl".into())
                .or_insert(LoggingLevel::Debug);
        }
        self
    }
}
