use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// API server configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The network interface and port to bind the API server onto.
    #[serde(default = "Config::default_bind")]
    pub bind: String,

    /// The health checks refresh frequency (in seconds).
    #[serde(default = "Config::default_healthcheck_refresh")]
    pub healthcheck_refresh: u64,

    /// The number of request handling threads.
    #[serde(default)]
    pub threads_count: Option<usize>,

    /// API server timeouts.
    #[serde(default)]
    pub timeouts: Timeouts,

    /// Configure TLS (for HTTPS) certificates.
    #[serde(default)]
    pub tls: Option<TlsConfig>,

    /// Enable/disable entire API trees.
    #[serde(default)]
    pub trees: APITrees,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: Config::default_bind(),
            healthcheck_refresh: 10,
            threads_count: None,
            timeouts: Timeouts::default(),
            tls: None,
            trees: APITrees::default(),
        }
    }
}

impl Config {
    fn default_bind() -> String {
        String::from("127.0.0.1:16016")
    }

    fn default_healthcheck_refresh() -> u64 {
        10
    }
}

/// Enable/disable entire API trees.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct APITrees {
    /// Enable/disable the introspection APIs.
    #[serde(default = "APITrees::default_true")]
    pub introspect: bool,

    /// Enable/disable the unstable API.
    #[serde(default = "APITrees::default_true")]
    pub unstable: bool,
}

impl Default for APITrees {
    fn default() -> APITrees {
        APITrees {
            introspect: true,
            unstable: true,
        }
    }
}

impl APITrees {
    fn default_true() -> bool {
        true
    }
}

// We can's fulfill the wish of the implicit-hasher clippy because
// we do not use the genieric hasher parameter in any LOCAL type.
#[allow(clippy::implicit_hasher)]
impl From<APITrees> for HashMap<&'static str, bool> {
    fn from(trees: APITrees) -> HashMap<&'static str, bool> {
        let mut flags = HashMap::new();
        flags.insert("introspect", trees.introspect);
        flags.insert("unstable", trees.unstable);
        flags
    }
}

/// API server timeouts.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Timeouts {
    /// Control the timeout, in seconds, for keep alive connections.
    #[serde(default = "Timeouts::default_keep_alive")]
    pub keep_alive: Option<usize>,

    /// Control the timeout, in seconds, for reads on existing connections.
    #[serde(default = "Timeouts::default_read")]
    pub read: Option<u64>,

    /// Control the timeout, in seconds, for writes on existing connections.
    #[serde(default = "Timeouts::default_write")]
    pub write: Option<u64>,
}

impl Default for Timeouts {
    fn default() -> Timeouts {
        Timeouts {
            keep_alive: Self::default_keep_alive(),
            read: Self::default_read(),
            write: Self::default_write(),
        }
    }
}

impl Timeouts {
    fn default_keep_alive() -> Option<usize> {
        Some(5)
    }

    fn default_read() -> Option<u64> {
        Some(5)
    }

    fn default_write() -> Option<u64> {
        Some(1)
    }
}

/// TLS (for HTTPS) certificates configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to a PEM bundle of trusted CAs for client authentication.
    #[serde(default)]
    pub clients_ca_bundle: Option<String>,

    /// Path to a PEM file with the server's public certificate.
    pub server_cert: String,

    /// Path to a PEM file with the server's PRIVATE certificate.
    pub server_key: String,
}
