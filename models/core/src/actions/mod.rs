use serde::Deserialize;
use serde::Serialize;

pub mod node;
pub mod orchestrator;

/// Approval requirements for action scheduling.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionApproval {
    /// Approval is granted and the action can be scheduled.
    #[serde(rename = "granted", alias = "GRANTED", alias = "Granted")]
    Granted,

    /// Approval from a user is required and the action CANNOT be scheduled yet.
    #[default]
    #[serde(rename = "required", alias = "REQUIRED", alias = "Required")]
    Required,
}
