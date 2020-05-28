use serde::Deserialize;

/// Advanced configuration for crates in this project.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Crates {
    /// List of Cargo.toml files to ignore where looking for changed crates.
    #[serde(default)]
    pub ignored: Vec<String>,

    /// List of crates that are expected to be published to crates.io
    ///
    /// These should be listed in order of publish (to allow crates to depend on other
    /// crates listed before them and publish correctly).
    #[serde(default)]
    pub publish: Vec<CratesPublish>,

    /// List of Cargo.toml files that define workspaces to operate on in the repo.
    #[serde(default)]
    pub workspaces: Vec<String>,
}

/// Crate publishing configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct CratesPublish {
    /// Allow `cargo package` to fail as part of `replidev release check`.
    ///
    /// This is usefull for crates that depend on other crates which would be published
    /// before them, but are not available while checking.
    #[serde(default = "CratesPublish::default_may_fail")]
    pub may_fail_check: bool,

    /// Path to the Cargo.toml file that defines the crate to publish.
    pub path: String,
}

impl CratesPublish {
    fn default_may_fail() -> bool {
        false
    }
}
