/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub files: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { files: Vec::new() }
    }
}
