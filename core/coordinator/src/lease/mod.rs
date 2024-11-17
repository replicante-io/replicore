//! Leases are the building block for coordination in Replicante Core.
//!
//! Leases only have one primary handler, with possible secondary handlers.
//! They provide an event interface to notify users of changes to their state,
//! such as acquiring or loosing a lease.
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::task::JoinHandle;

mod control;

#[cfg(any(test, feature = "test-fixture"))]
pub mod fixture;

#[cfg(test)]
mod tests;

/// Operations implemented by Coordination Services supported by Replicante Core.
#[async_trait::async_trait]
pub trait ILease: Send {
    /// Relinquish Primary control of the lease, if it was held by this instance.
    async fn step_down(&mut self) -> Result<()>;

    /// Watch for changes to the lease reported by the coordination service.
    ///
    /// The watching logic is also responsible for any necessary keep-alive logic
    /// that may be needed by the implementing service.
    ///
    /// If the `candidate` flag is set, the watching logic is responsible for setting up the
    /// lease and run for election if it is not yet attempting to acquiring exclusive access.
    async fn watch(&mut self, candidate: bool) -> Result<State>;
}

/// Acquire, release and watch a coordination lease.
///
/// ## CPU Starvation Protection
///
/// To ensure lease keep-alive logic does not suffer from "polling starvation" when
/// used in combination with CPU intensive futures, the [`Lease`] will spawn a control task when
/// created and perform operations within it.
///
/// When the [`Lease`] is dropped the background task is cancelled.
pub struct Lease {
    /// Communication channels with the Lease control task.
    channels: self::control::ControlChannels,

    /// Unique identified for the lease.
    id: String,

    /// Keep track of the most recently seen state before changes.
    last_state: State,

    /// Tokio join handle for the task managing the lease.
    task: JoinHandle<()>,
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Lease {
    pub fn new<L, S>(id: S, lease: L) -> Self
    where
        L: ILease + 'static,
        S: Into<String>,
    {
        let id = id.into();
        let lease = Box::new(lease);

        let (channels, state) = self::control::ControlState::new(lease);
        let task = tokio::spawn(self::control::task(state));

        Lease {
            channels,
            id,
            last_state: State::Idle,
            task,
        }
    }

    /// Request the lease to be released and not re-acquired for at least `delay`.
    ///
    /// This is a no-op if the lease does not hold the primary role.
    pub async fn step_down(&self, delay: Duration) -> Result<()> {
        let (response_send, response) = tokio::sync::oneshot::channel();
        let command = self::control::ControlCommands::StepDown(delay, response_send);
        if self.channels.commands.send(command).await.is_err() {
            panic!("lease '{}' control task lost!", self.id);
        }
        match response.await {
            Err(_) => panic!("lease '{}' control task lost!", self.id),
            Ok(response) => response,
        }
    }

    /// Watch the lease for state changes.
    ///
    /// Watching the lease will return the most recent state change since the last check.
    /// There is no guarantee all state changes are visible to watchers.
    pub async fn watch(&mut self) -> Result<State> {
        // Loop over watch notifications to filter out duplicate states.
        loop {
            // Wait for a state change notification.
            let state = match self.channels.state.recv().await {
                Some(state) => state?,
                None => panic!("lease '{}' control task lost!", self.id),
            };

            // Skip notification if state has not changes.
            if self.last_state == state {
                continue;
            }

            // If we go from primary to another state we lost the lease so update to `Lost`.
            let mut state = state;
            if matches!(self.last_state, State::Primary) && !matches!(state, State::Primary) {
                state = State::Lost;
            }

            self.last_state = state;
            return Ok(state);
        }
    }
}

/// Current state of a lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    /// The lease is a candidate for future elections.
    Candidate,

    /// The lease is currently held and the exclusive logic can be executed.
    Primary,

    /// The lease is already heal elsewhere and the exclusive logic must not be executed.
    Secondary,

    /// The lease was in `Primary` but was lost and execution of the exclusive logic should stop.
    Lost,

    /// The lease paused and not trying to obtain exclusive access.
    Idle,
}

impl State {
    /// Check if the lease `State` is primary or not.
    pub fn is_primary(&self) -> bool {
        matches!(self, State::Primary)
    }
}
