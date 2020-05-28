use serde::Deserialize;

/// Advance configuration for npm packages in this project.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Npm {
    /// List of package.json files to ignore where looking for changed npm packages.
    #[serde(default)]
    pub ignored: Vec<String>,

    /// List of package.json files that define workspaces to operate on in the repo.
    #[serde(default)]
    pub workspaces: Vec<String>,
}
