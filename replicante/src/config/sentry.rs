/// Sentry API response capture filter.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SentryCaptureApi {
    #[serde(rename = "no")]
    No,

    #[serde(rename = "client")]
    Client,

    #[serde(rename = "server")]
    Server,
}

impl Default for SentryCaptureApi {
    fn default() -> SentryCaptureApi {
        SentryCaptureApi::Server
    }
}

/// Sentry integration configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Sentry API response capture filter.
    #[serde(default)]
    pub capture_api_errors: SentryCaptureApi,

    /// The DSN to use to configure sentry.
    pub dsn: String,
}
