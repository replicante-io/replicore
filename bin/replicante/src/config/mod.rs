use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_logging::Config as LoggingConfig;
use replicante_logging::LoggingLevel;
use replicante_models_core::scope::Namespace;
use replicante_models_core::scope::NsHttpsTransport;
use replicante_service_coordinator::Config as CoordinatorConfig;
use replicante_service_tasks::Config as TasksConfig;
use replicante_util_tracing::Config as TracingConfig;

use crate::components::DiscoveryConfig;
use crate::interfaces::api::Config as APIConfig;
use crate::ErrorKind;
use crate::Result;

mod components;
mod events;
mod sentry;
mod storage;
mod task_workers;
mod timeouts;

pub use self::components::ComponentsConfig;
pub use self::events::EventsConfig;
pub use self::events::SnapshotsConfig as EventsSnapshotsConfig;
pub use self::sentry::SentryCaptureApi;
pub use self::sentry::SentryConfig;
pub use self::storage::StorageConfig;
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

    /// Settings that will move to the DB once namespaces are fully introduced.
    #[serde(default)]
    pub tmp_namespace_settings: TmpNsSettings,
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

    #[cfg(test)]
    pub fn mock() -> Config {
        Config::from_reader(include_str!("mock_config.yaml").as_bytes())
            .expect("mock config to load")
    }
}

/// Settings that will move to the DB once namespaces are fully introduced.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TmpNsSettings {
    /// HTTPS configuration for agent transport.
    #[serde(default)]
    pub https_transport: TmpNsSettingsHttps,
}

impl From<TmpNsSettings> for Namespace {
    fn from(config: TmpNsSettings) -> Namespace {
        Namespace {
            ns_id: "tmp_global_namespace".into(),
            https_transport: NsHttpsTransport {
                ca_bundle: config.https_transport.ca_bundle,
                client_key_id: config.https_transport.client_key,
            },
        }
    }
}

/// HTTPS configuration for agent transport.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TmpNsSettingsHttps {
    /// PEM formatted bundle of CA certificates to validate agent certificates.
    pub ca_bundle: Option<String>,

    /// Path to a PEM formatted HTTPS client **private** key.
    pub client_key: Option<String>,
}
