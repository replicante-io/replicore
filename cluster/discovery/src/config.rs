use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Map;
use serde_json::Value;

/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub files: Vec<String>,

    #[serde(default)]
    pub http: Vec<HttpConfig>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            files: Vec::new(),
            http: Vec::new(),
        }
    }
}

/// HTTP cluster discovery configurations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Optional JSON object to used as the body in HTTP requests.
    #[serde(default)]
    pub body: Option<Map<String, Value>>,

    /// Optional headers to be added to HTTP requests.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// HTTP Requests timeout (in milliseconds).
    #[serde(default = "HttpConfig::default_timeout")]
    pub timeout: u64,

    /// HTTP Client TLS configuration.
    #[serde(default)]
    pub tls: HttpTlsConfig,

    /// URL of to fetch clusters from.
    pub url: String,
}

impl PartialEq for HttpConfig {
    fn eq(&self, other: &HttpConfig) -> bool {
        self.headers == other.headers && self.tls == other.tls && self.url == other.url
    }
}

impl Eq for HttpConfig {}

impl Hash for HttpConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.headers.hash(state);
        self.tls.hash(state);
        self.url.hash(state);
    }
}

impl HttpConfig {
    fn default_timeout() -> u64 {
        30_000
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct HttpTlsConfig {
    /// Optional path to a CA certificates bundle to validate servers with.
    #[serde(default)]
    pub ca_cert: Option<String>,

    /// Optional path to an HTTP client TLS certificate.
    #[serde(default)]
    pub client_cert: Option<String>,
}

impl Default for HttpTlsConfig {
    fn default() -> Self {
        HttpTlsConfig {
            ca_cert: None,
            client_cert: None,
        }
    }
}
