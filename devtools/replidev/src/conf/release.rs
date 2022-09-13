use serde::Deserialize;

use super::Image;
use super::VersionFrom;

/// Rule to extract binaries from a container image.
#[derive(Clone, Debug, Deserialize)]
pub struct ExtractBinary {
    /// The kind of entity to extract from the container image.
    pub extract: ExtractBinaryMode,

    /// The path to the entity to extract.
    pub path: String,

    /// Container image registry to add to image tags.
    #[serde(default = "Image::default_registry")]
    pub registry: String,

    /// Container image repository to add to image tags.
    pub repo: String,

    /// Optional override of the extracted file name.
    #[serde(default)]
    pub target_name: Option<String>,

    /// How to determine the version tag.
    pub version: VersionFrom,
}

/// The kind of entity to extract from the container image.
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum ExtractBinaryMode {
    /// Extract a directory as a tar archive.
    #[serde(rename = "directory")]
    Directory,

    /// Extract a file as it is.
    #[serde(rename = "file")]
    File,
}

/// How to determine the git tag generated by `replidev release publish`.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "from")]
pub enum ReleaseTag {
    /// Use the version from a Cargo.toml file to generate the tag.
    #[serde(rename = "cargo")]
    Cargo {
        /// Path to the Cargo.toml file to use.
        path: String,
    },

    /// Use the current date to generate the tag.
    #[serde(rename = "date")]
    Date,

    /// Use the version from a package.json file to generate the tag.
    #[serde(rename = "npm")]
    Npm {
        /// Path to the package.json file to use.
        path: String,
    },
}
