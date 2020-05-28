use serde::Deserialize;

/// Definition of how to build an image for the project.
#[derive(Clone, Debug, Deserialize)]
pub struct Image {
    /// Path to the build context for the image.
    #[serde(default = "Image::default_context")]
    pub context: String,

    /// Path to the Dockerfile to use for the build.
    #[serde(default)]
    pub dockerfile: Option<String>,

    /// Name of the image, mainly used for output progress.
    pub name: String,

    /// Container image registry to add to image tags.
    #[serde(default = "Image::default_registry")]
    pub registry: String,

    /// Container image repository to add to image tags.
    pub repo: String,

    /// How to determine the version tag.
    pub version: VersionFrom,
}

impl Image {
    fn default_context() -> String {
        ".".to_string()
    }

    pub(crate) fn default_registry() -> String {
        "replicanteio".to_string()
    }
}

/// Supported methods to determine the image version.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "from")]
pub enum VersionFrom {
    /// Fetch the current version from a Cargo.toml file.
    #[serde(rename = "cargo")]
    Cargo {
        /// Path to the Cargo.toml file to use.
        path: String,
    },

    /// Fetch the current version from a package.json file.
    #[serde(rename = "npm")]
    Npm {
        /// Path to the package.json file to use.
        path: String,
    },
}
