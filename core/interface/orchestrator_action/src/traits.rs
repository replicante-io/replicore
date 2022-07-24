use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;

use crate::ProgressChanges;

/// Orchestrator actions implement this trait to describe and execute actions.
///
/// Implementations of `OrchestratorAction`s have to be `Send` and `Sync` as
/// action progression can be invoked by any number of threads concurrently.
pub trait OrchestratorAction: Send + Sync {
    /// Start or progress execution of the action.
    ///
    /// Progression logic is given the `OrchestratorAction` model created by API calls,
    /// generally as a result of user requests.
    ///
    /// Only some aspects of this record can be changed by the implementation by
    /// returning the new values in a `ProgressChanges` object.
    /// This ensures that action implementations can't break integrity of the system.
    /// For example an implementation is not allowed to change the action `kind` or calling `args`.
    ///
    /// # Errors
    ///
    /// If the action implementation fails for any reason it can return an error to indicate so.
    /// On error the action is automatically transitioned to the final `Failed` state.
    /// The error information is encoded and store in the `state_payload` attribute for users.
    ///
    /// Retry of failed actions is NOT automatically handled so transient errors may be better
    /// handled by the implementation.
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>>;
}
