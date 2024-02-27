use serde::Deserialize;
use serde::Serialize;

/// Sentry integration configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Sentry API response capture filter.
    #[serde(default)]
    pub capture_api_errors: bool,

    /// The DSN to use to configure sentry.
    pub dsn: String,
}
