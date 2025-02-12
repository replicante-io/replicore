//! Leases are the building block for coordination in Replicante Core.
//!
//! Leases only have one primary handler, with possible secondary handlers.
//! They provide an event interface to notify users of changes to their state,
//! such as acquiring or loosing a lease.
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;
use tokio::task::JoinHandle;

use replicore_context::Context;

mod control;

#[cfg(test)]
mod tests;

/// Operations implemented by Coordination Services supported by Replicante Core.
#[async_trait::async_trait]
pub trait ILease: Send {
    /// Relinquish Primary control of the lease, if it was held by this instance.
    async fn step_down(&mut self, context: &Context) -> Result<()>;

    /// Watch for changes to the lease reported by the coordination service.
    ///
    /// The watching logic is also responsible for any necessary keep-alive logic
    /// that may be needed by the implementing service.
    ///
    /// If the `candidate` flag is set, the watching logic is responsible for setting up the
    /// lease and run for election if it is not yet attempting to acquiring exclusive access.
    async fn watch(&mut self, context: &Context, candidate: bool) -> Result<State>;
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

    /// Handle to step down the lease or create new handles.
    handle: LeaseHandle,

    /// Watch channel sender to update the state seen by [`LeaseHandle`]s.
    handle_updater: Sender<State>,

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
    /// Create a builder to enable advanced lease features.
    pub fn builder<L, S>(context: Context, id: S, lease: L) -> LeaseBuilder
    where
        L: ILease + 'static,
        S: Into<String>,
    {
        LeaseBuilder::new(context, id, lease)
    }

    /// Return the ID of the lease.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return a handle to inspect and control the [`Lease`].
    pub fn handle(&self) -> LeaseHandle {
        self.handle.clone()
    }

    /// Create a lease and begin attempts to acquire it immediately.
    pub fn new<L, S>(context: Context, id: S, lease: L) -> Self
    where
        L: ILease + 'static,
        S: Into<String>,
    {
        Self::builder(context, id, lease).build()
    }

    /// Request the lease to be released and not re-acquired for at least `delay`.
    ///
    /// This is a no-op if the lease does not hold the primary role.
    pub async fn step_down(&self, delay: Duration) -> Result<()> {
        self.handle.step_down(delay).await
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

            let _ = self.handle_updater.send(state);
            self.last_state = state;
            return Ok(state);
        }
    }
}

/// Incrementally build a [`Lease`] without starting the control task until the end.
pub struct LeaseBuilder {
    /// Communication channels with the Lease control task.
    channels: self::control::ControlChannels,

    /// Operation context sent to the control task when it is started.
    context: Context,

    /// Handle to create new lease handles from.
    handle: LeaseHandle,

    /// Watch channel sender to update the state seen by [`LeaseHandle`]s.
    handle_updater: Sender<State>,

    /// Unique identified for the lease.
    id: String,

    /// Lease control state sent to the control task when it is started.
    state: self::control::ControlState,
}

impl LeaseBuilder {
    fn new<L, S>(context: Context, id: S, lease: L) -> LeaseBuilder
    where
        L: ILease + 'static,
        S: Into<String>,
    {
        // Prepare lease control elements.
        let id = id.into();
        let lease = Box::new(lease);
        let (channels, state) = self::control::ControlState::new(lease);

        // Prepare lease handling elements.
        let (handle_updater, handle_watcher) = tokio::sync::watch::channel(State::Idle);
        let handle = LeaseHandle {
            commands: channels.commands.clone(),
            id: id.clone(),
            state: handle_watcher,
        };

        // Collect everything needed to build a lease.
        LeaseBuilder {
            channels,
            context,
            handle,
            handle_updater,
            id,
            state,
        }
    }

    /// Build a [`Lease`] and begin attempts to acquire it immediately.
    pub fn build(self) -> Lease {
        let task = tokio::spawn(self::control::task(self.context, self.state));
        Lease {
            channels: self.channels,
            handle: self.handle,
            handle_updater: self.handle_updater,
            id: self.id,
            last_state: State::Idle,
            task,
        }
    }

    /// Return a handle to inspect and control the [`Lease`] once it is built.
    pub fn handle(&self) -> LeaseHandle {
        self.handle.clone()
    }
}

/// Handle to inspect and step down a [`Lease`] without requiring ownership of it.
#[derive(Clone)]
pub struct LeaseHandle {
    /// Channel to send control commands to the lease managing task.
    commands: tokio::sync::mpsc::Sender<self::control::ControlCommands>,

    /// Unique identified for the lease.
    id: String,

    /// Receive state updates and store the most recent state.
    state: Receiver<State>,
}

impl LeaseHandle {
    /// Get the identifier of the corresponding [`Lease`].
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the currently known state of the corresponding [`Lease`].
    pub fn state(&self) -> State {
        *self.state.borrow()
    }

    /// Request the lease to be released and not re-acquired for at least `delay`.
    ///
    /// This is a no-op if the lease does not hold the primary role.
    pub async fn step_down(&self, delay: Duration) -> Result<()> {
        let (response_send, response) = tokio::sync::oneshot::channel();
        let command = self::control::ControlCommands::StepDown(delay, response_send);
        if self.commands.send(command).await.is_err() {
            panic!("lease '{}' control task lost", self.id);
        }
        match response.await {
            Err(_) => panic!("lease '{}' control task lost", self.id),
            Ok(response) => response,
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

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Candidate => write!(f, "CANDIDATE"),
            Self::Primary => write!(f, "PRIMARY"),
            Self::Secondary => write!(f, "SECONDARY"),
            Self::Lost => write!(f, "LOST"),
            Self::Idle => write!(f, "IDLE"),
        }
    }
}

impl State {
    /// Check if the lease `State` is primary or not.
    pub fn is_primary(&self) -> bool {
        matches!(self, State::Primary)
    }
}
