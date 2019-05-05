use std::collections::HashMap;

/// API server configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The network interface and port to bind the API server onto.
    #[serde(default = "Config::default_bind")]
    pub bind: String,

    /// The number of request handling threads.
    #[serde(default)]
    pub threads_count: Option<usize>,

    /// API server timeouts.
    #[serde(default)]
    pub timeouts: Timeouts,

    /// Enable/disable entire API trees.
    #[serde(default)]
    pub trees: APITrees,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: Config::default_bind(),
            threads_count: None,
            timeouts: Timeouts::default(),
            trees: APITrees::default(),
        }
    }
}

impl Config {
    /// Default value for `bind` used by serde.
    fn default_bind() -> String { String::from("127.0.0.1:16016") }
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
    pub keep_alive: Option<u64>,

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
            keep_alive: Some(5),
            read: Some(5),
            write: Some(1),
        }
    }
}

impl Timeouts {
    fn default_keep_alive() -> Option<u64> {
        Some(5)
    }

    fn default_read() -> Option<u64> {
        Some(5)
    }

    fn default_write() -> Option<u64> {
        Some(1)
    }
}
