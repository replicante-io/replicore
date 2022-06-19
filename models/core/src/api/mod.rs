use serde::Deserialize;
use serde::Serialize;

pub mod apply;
pub mod discovery_settings;
pub mod objects;
pub mod orchestrator_action;
pub mod validate;

/// Replicante version information.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Version {
    pub commit: String,
    pub taint: String,
    pub version: String,
}
