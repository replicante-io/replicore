use std::fs::File;

use failure::ResultExt;
use serde::Deserialize;
use serde_yaml::Mapping;
use serde_yaml::Value;

use crate::ErrorKind;
use crate::Result;

const CONF_FILE: &str = "replidev.yaml";
const CONF_FILE_LOCAL: &str = "replidev.local.yaml";

/// Project specific configuration.
#[derive(Debug, Deserialize)]
pub struct Conf {
    /// Current project to operate on.
    pub project: Project,

    /// Command to execute podman.
    #[serde(default = "Conf::default_podman")]
    pub podman: String,
}

impl Conf {
    /// Load the local project's configuration file.
    pub fn from_file() -> Result<Self> {
        // Read config file and optional override file.
        let base = Conf::load_file(CONF_FILE)?;
        let local = if std::path::Path::new(CONF_FILE_LOCAL).exists() {
            Conf::load_file(CONF_FILE_LOCAL)?
        } else {
            Mapping::new()
        };

        // Merge the config options and decode the result.
        let conf = Conf::merge(base, local);
        let conf = serde_yaml::from_value(conf).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
    }
}

impl Conf {
    fn default_podman() -> String {
        "podman".into()
    }

    fn load_file(file: &str) -> Result<Mapping> {
        let conf = File::open(file).with_context(|_| ErrorKind::ConfigLoad)?;
        let conf = serde_yaml::from_reader(conf).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
    }

    fn merge(mut base: Mapping, local: Mapping) -> Value {
        for (key, value) in local {
            base.insert(key, value);
        }
        Value::Mapping(base)
    }
}

/// Supported replidev projects.
#[derive(PartialEq, Eq, Debug, Deserialize)]
pub enum Project {
    /// Replicante Core
    #[serde(rename = "core")]
    Core,

    /// Replicante Playgrounds Projects
    #[serde(rename = "playground")]
    Playground,
}

impl Project {
    /// Check if a project is allowed to execute the `deps` family of commands.
    pub fn allow_deps(&self) -> bool {
        *self == Self::Core
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Core => write!(fmt, "core"),
            Self::Playground => write!(fmt, "playground"),
        }
    }
}
