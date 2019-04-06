use std::collections::HashMap;

/// API server configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_bind")]
    pub bind: String,

    /// Enable/disable entire API trees.
    #[serde(default)]
    pub trees: APITrees,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: Config::default_bind(),
            trees: APITrees::default(),
        }
    }
}

impl Config {
    /// Default value for `bind` used by serde.
    fn default_bind() -> String { String::from("127.0.0.1:16016") }
}

/// Enable/disable entire API trees.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
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

impl From<APITrees> for HashMap<&'static str, bool> {
    fn from(trees: APITrees) -> HashMap<&'static str, bool> {
        let mut flags = HashMap::new();
        flags.insert("introspect", trees.introspect);
        flags.insert("unstable", trees.unstable);
        flags
    }
}
