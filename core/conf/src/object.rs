//! Data object storing replicore's configuration.
use serde::Deserialize;
use serde::Serialize;

use replisdk::runtime::actix_web::ServerConfig;
use replisdk::runtime::telemetry::TelemetryConfig;

use replicore_tasks::conf::TasksExecutorConf;

use super::ExclusivesConf;
use super::RuntimeConf;

/// Global configuration for the Replicante Core process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Distributed Coordinator service configuration.
    pub coordinator: BackendConf,

    /// Events Streaming Platform service configuration.
    pub events: BackendConf,

    /// Control Plane exclusive tasks.
    #[serde(default)]
    pub exclusives: ExclusivesConf,

    /// HTTP Server configuration.
    #[serde(default)]
    pub http: HttpConf,

    /// Process runtime configuration.
    #[serde(default)]
    pub runtime: RuntimeConf,

    /// Persistent Store service configuration.
    pub store: BackendConf,

    /// Configuration for background tasks execution and backend service.
    pub tasks: TasksConf,

    /// Telemetry configuration for the process.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

/// Unstructured configuration for runtime selected service backends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackendConf {
    /// ID of the backend selected to provide the service.
    pub backend: String,

    /// Backend specific configuration options.
    #[serde(default, flatten)]
    pub options: serde_json::Value,
}

/// HTTP Server configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HttpConf {
    /// Enable API endpoints to inspect and manage the exclusive tasks lease.
    #[serde(default = "HttpConf::default_control_exclusives_lease")]
    pub control_exclusives_lease: bool,

    /// HTTP Server configuration.
    #[serde(flatten, default)]
    pub server: ServerConfig,
}

impl HttpConf {
    fn default_control_exclusives_lease() -> bool {
        false
    }
}

impl Default for HttpConf {
    fn default() -> Self {
        HttpConf {
            control_exclusives_lease: HttpConf::default_control_exclusives_lease(),
            server: ServerConfig::default(),
        }
    }
}

/// Configuration for background tasks execution and backend service.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TasksConf {
    /// Background Tasks service configuration.
    pub service: BackendConf,

    /// Background tasks executor configuration.
    #[serde(default)]
    pub executor: TasksExecutorConf,
}
