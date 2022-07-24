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
pub struct ProgressChanges {
    pub state: OrchestratorActionState,
    pub state_payload: Option<serde_json::Value>,
}
