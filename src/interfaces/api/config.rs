/// API server configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "Config::default_bind")]
    pub bind: String
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: Config::default_bind(),
        }
    }
}

impl Config {
    /// Default value for `bind` used by serde.
    fn default_bind() -> String { String::from("127.0.0.1:16016") }
}
