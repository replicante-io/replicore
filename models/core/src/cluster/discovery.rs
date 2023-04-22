use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

/// Select one of the supported discovery backends.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "backend")]
pub enum DiscoveryBackend {
    /// HTTP Endpoint discovery.
    #[serde(rename = "http")]
    Http(HttpDiscovery),
}

/// Cluster discovery settings for a single discovery backend.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DiscoverySettings {
    /// Backend to discover clusters from.
    #[serde(flatten)]
    pub backend: DiscoveryBackend,

    /// Enable or disable discovery against this backend.
    #[serde(default = "DiscoverySettings::default_enabled")]
    pub enabled: bool,

    /// Interval, in seconds, between discovery runs.
    pub interval: i64,

    /// Namespace unique name for this discovery settings.
    pub name: String,

    /// Namespace the discovery settings belongs to.
    pub namespace: String,
}

impl DiscoverySettings {
    fn default_enabled() -> bool {
        true
    }

    /// Create a `DiscoverySettings` from an apply API object.
    pub fn from_object(
        namespace: String,
        name: String,
        settings: crate::api::objects::DiscoverySettings,
    ) -> DiscoverySettings {
        DiscoverySettings {
            backend: settings.backend,
            enabled: settings.enabled,
            interval: settings.interval,
            name,
            namespace,
        }
    }
}

/// HTTP cluster discovery configurations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpDiscovery {
    /// Optional JSON object to used as the body in HTTP requests.
    #[serde(default)]
    pub body: Option<Map<String, Value>>,

    /// Optional headers to be added to HTTP requests.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// HTTP method to send the request as.
    #[serde(default)]
    pub method: HttpRequestMethod,

    /// HTTP Requests timeout (in milliseconds).
    #[serde(default = "HttpDiscovery::default_timeout")]
    pub timeout: u64,

    /// HTTP Client TLS configuration.
    #[serde(default)]
    pub tls: HttpTlsConfig,

    /// URL of to fetch clusters from.
    pub url: String,
}

impl PartialEq for HttpDiscovery {
    fn eq(&self, other: &HttpDiscovery) -> bool {
        self.headers == other.headers && self.tls == other.tls && self.url == other.url
    }
}

impl Eq for HttpDiscovery {}

impl Hash for HttpDiscovery {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.headers.hash(state);
        self.tls.hash(state);
        self.url.hash(state);
    }
}

impl HttpDiscovery {
    fn default_timeout() -> u64 {
        3_000
    }
}

/// HTTP Method to use when sending requests.
///
/// This impacts the use of pagination and body, which are only possible with POST requests.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum HttpRequestMethod {
    #[serde(rename = "GET")]
    Get,

    #[default]
    #[serde(rename = "POST")]
    Post,
}

/// TLS configuration used to connect to the remote server.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct HttpTlsConfig {
    /// Optional path to a CA certificates bundle to validate servers with.
    #[serde(default)]
    pub ca_cert: Option<String>,

    /// Optional path to an HTTP client TLS certificate.
    #[serde(default)]
    pub client_cert: Option<String>,
}
