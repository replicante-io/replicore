use std::fs::File;

use failure::ResultExt;
use serde::Deserialize;

use crate::ErrorKind;
use crate::Result;

const CONF_FILE: &str = "replidev.yaml";

/// Project specific configuration.
#[derive(Debug, Deserialize)]
pub struct Conf {
    /// Current project to operate on.
    pub project: Project,
}

impl Conf {
    /// Load the local project's configuration file.
    pub fn from_file() -> Result<Self> {
        let conf = File::open(CONF_FILE).with_context(|_| ErrorKind::ConfigLoad)?;
        let conf = serde_yaml::from_reader(conf).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
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

impl std::fmt::Display for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Core => write!(fmt, "core"),
            Self::Playground => write!(fmt, "playground"),
        }
    }
}
