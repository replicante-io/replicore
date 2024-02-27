use serde::Deserialize;
use serde::Serialize;

pub use replicante_models_core::actions::orchestrator::OrchestratorActionMetadata;

use replicante_models_core::actions::orchestrator::OrchestratorActionState;

pub mod errors;
mod registry;
mod traits;

pub use self::errors::ActionAlreadyRegistered;
pub use self::registry::OrchestratorActionRegistry;
pub use self::registry::OrchestratorActionRegistryBuilder;
pub use self::registry::OrchestratorActionRegistryEntry;
pub use self::traits::OrchestratorAction;

#[cfg(feature = "test-api")]
pub use self::registry::TestRegistryClearGuard;

/// Store changes made to an `OrchestratorAction` model as a result of progressing the action.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ProgressChanges {
    /// New state of the action.
    pub state: OrchestratorActionState,

    /// If set, updates the action state payload.
    pub state_payload: Option<Option<serde_json::Value>>,

    /// If set, updates the action state error payload.
    pub state_payload_error: Option<Option<serde_json::Value>>,
}
