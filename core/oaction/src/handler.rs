//! Interface for implementation of orchestrator action execution handling.
use anyhow::Result;

use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionState;

use replicore_context::Context;

/// Describes how state data should be changed after an [`OActionHandler::invoke`] call.
#[derive(Debug, Default)]
pub enum OActionChangeValue {
    /// Remove the current state data.
    Remove,

    /// No changes should be made to state data.
    #[default]
    Unchanged,

    /// Update the state data to the given value.
    Update(serde_json::Value),
}

/// Changes to an [`OAction`] record as a result of its [`OActionHandler`] invocation.
pub struct OActionChanges {
    /// Optionally change the action error data.
    pub error: OActionChangeValue,

    /// Optionally change the action payload data.
    pub payload: OActionChangeValue,

    /// Set the [`OAction`] state after the handler is invoked.
    pub state: OActionState,
}

impl OActionChanges {
    /// Update or reset the orchestrator action error data.
    pub fn error<E>(mut self, error: E) -> Self
    where
        E: Into<Option<serde_json::Value>>,
    {
        self.error = match error.into() {
            Some(error) => OActionChangeValue::Update(error),
            None => OActionChangeValue::Remove,
        };
        self
    }

    /// Update or reset the orchestrator action payload data.
    pub fn payload<P>(mut self, payload: P) -> Self
    where
        P: Into<Option<serde_json::Value>>,
    {
        self.payload = match payload.into() {
            Some(payload) => OActionChangeValue::Update(payload),
            None => OActionChangeValue::Remove,
        };
        self
    }

    /// Update the orchestrator action state as a result of an invocation.
    pub fn to(state: OActionState) -> OActionChanges {
        OActionChanges {
            error: Default::default(),
            payload: Default::default(),
            state,
        }
    }
}

/// Interface for orchestrator action logic to progress an [`OAction`] records.
#[async_trait::async_trait]
pub trait OActionHandler: std::fmt::Debug + Send + Sync {
    /// Execute action specific logic to move an [`OAction`] towards a final state.
    ///
    /// [`OAction`] records track the current recorded state of an action execution
    /// and are provided to the invoke method to determine what the next steps are.
    ///
    /// When the [`OAction::state`] is `PENDING_SCHEDULE` the execution has never executed yet.
    /// In this case the [`OAction`] is updated to the `RUNNING` phase if the method
    /// returns no change details, otherwise the details are used to update the action.
    ///
    /// ## Errors
    ///
    /// If the invocation fails for any reason it can return an error to indicate so.
    /// On error the [`OAction`] is updated to the final `FAILED` state.
    /// The error information is stored in the [`OAction::state_payload_error`] for user review.
    ///
    /// Retry of failed actions is NOT automatically handled so transient failures need
    /// to be handled by the implementation if needed.
    async fn invoke(
        &self,
        context: &Context,
        action: &OAction,
    ) -> Result<OActionChanges>;
}
