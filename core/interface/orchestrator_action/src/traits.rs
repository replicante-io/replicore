use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_store_primary::store::Store;

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

    /// Workaround for access to Primary store.
    ///
    /// Some actions will need access to the Primary Store.
    /// While a long term solution (likely involving a global singleton) is yet to iron out
    /// and implement this method is called before an action is progressed and allows
    /// implementations to keep track of the store instance.
    ///
    /// As a guide, the suggestion is to use a `RwLock` combined with an `AtomicBool`:
    ///
    /// * The implementation will check initialisation with the `AtomicBool`.
    /// * If the `AtomicBool` is false:
    ///   * The `RwLock` is acquired for write and the store cloned. 
    ///   * The `AtomicBool` is set to true to avoid further requests for a write lock.
    ///
    /// The store instance is the accessed by requesting a read lock and expected always present.
    fn set_store(&self, _store: &Store) {}
}
