/// API server configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_bind")]
    pub bind: String,

    /// Enable/disable entire API versions.
    ///
    /// Useful for advanced operators that which to control access to experimental
    /// or legacy versions that are supported by the current version.
    ///
    /// Example use cases are:
    ///
    ///   * Upgrade prep: testing new API versions while having a quick rollback plan.
    ///   * Controlled rollout: be prepared for when verions are no longer supported.
    #[serde(default)]
    pub versions: APIVersions,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: Config::default_bind(),
            versions: APIVersions::default(),
        }
    }
}

impl Config {
    /// Default value for `bind` used by serde.
    fn default_bind() -> String { String::from("127.0.0.1:16016") }
}

/// Enable/disable entire API versions.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct APIVersions {
    /// Enable/disable the unstable API.
    ///
    /// The unstable API version is for endpoints in the early development cycle
    /// where the attributes and parameters can change a lot and often.
    #[serde(default = "APIVersions::default_true")]
    pub unstable: bool,
}

impl Default for APIVersions {
    fn default() -> APIVersions {
        APIVersions {
            unstable: true,
        }
    }
}

impl APIVersions {
    fn default_true() -> bool {
        true
    }
}
