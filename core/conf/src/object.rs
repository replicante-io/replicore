//! Data object storing replicore's configuration.
use serde::Deserialize;
use serde::Serialize;

use replisdk::runtime::actix_web::ServerConfig;
use replisdk::runtime::telemetry::TelemetryConfig;

use super::RuntimeConf;

/// Global configuration for the Replicante Core process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Conf {
    /// HTTP Server configuration.
    #[serde(default)]
    pub http: ServerConfig,

    /// Process runtime configuration.
    #[serde(default)]
    pub runtime: RuntimeConf,

    /// Telemetry configuration for the process.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}
