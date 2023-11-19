//! Data object storing replicore's configuration.
use serde::Deserialize;
use serde::Serialize;

use replisdk::runtime::actix_web::ServerConfig;
use replisdk::runtime::telemetry::TelemetryConfig;

use super::RuntimeConf;

/// Global configuration for the Replicante Core process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// Events Streaming Platform service configuration.
    pub events: BackendConf,

    /// HTTP Server configuration.
    #[serde(default)]
    pub http: ServerConfig,

    /// Process runtime configuration.
    #[serde(default)]
    pub runtime: RuntimeConf,

    /// Persistent Store service configuration.
    pub store: BackendConf,

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
