pub use replicante_models_core::actions::orchestrator::OrchestratorActionMetadata;

mod error;
mod registry;
mod traits;

pub use self::error::ActionAlreadyRegistered;
pub use self::registry::OrchestratorActionRegistry;
pub use self::registry::OrchestratorActionRegistryBuilder;
pub use self::registry::OrchestratorActionRegistryEntry;
pub use self::traits::OrchestratorAction;

#[cfg(feature = "test-api")]
pub use self::registry::TestRegistryClearGuard;
