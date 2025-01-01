//! Dedicated module for the lease control background task.
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Sender as SendOnce;
use tokio::time::Sleep;

use replicore_context::Context;

use super::ILease;
use super::State;

/// Communication channels with the Lease control task.
pub struct ControlChannels {
    /// Channel to send control commands to the lease managing task.
    pub commands: Sender<ControlCommands>,

    /// Channel to receive lease state change notifications.
    pub state: Receiver<Result<State>>,
}

/// Commands to control the lease managing task.
pub enum ControlCommands {
    /// Ask a primary lease holder to relinquish control.
    StepDown(Duration, SendOnce<Result<()>>),
}

/// Collect mutable state for the lease control task.
pub struct ControlState {
    /// Indicates the lease wants to run for election.
    candidate: bool,

    /// Channel to receive control commands from the lease holder.
    commands: Receiver<ControlCommands>,

    /// Lease backend implementation to watch/control the lease with.
    lease: Box<dyn ILease>,

    /// Watch channel to notify lease holders of state changes.
    states: Sender<Result<State>>,

    /// A sleep notification to be set and selected during the step down period.
    step_down_delay: Pin<Box<Sleep>>,

    /// Indicate the step_down_delay is active and should be `tokio::selected`.
    step_down_delay_used: bool,
}

impl ControlState {
    pub fn new(lease: Box<dyn ILease>) -> (ControlChannels, ControlState) {
        let (commands_tx, commands_rx) = tokio::sync::mpsc::channel(1);
        let (states_tx, states_rx) = tokio::sync::mpsc::channel(20);

        let channels = ControlChannels {
            commands: commands_tx,
            state: states_rx,
        };
        let state = ControlState {
            candidate: true,
            commands: commands_rx,
            lease,
            states: states_tx,
            // We conditionally await within a `tokio::select!` but we must always have a future.
            // So set up a useless sleep to initialise the box with.
            step_down_delay: Box::pin(tokio::time::sleep(Duration::from_millis(1))),
            step_down_delay_used: false,
        };

        (channels, state)
    }
}

/// Background task that manages a lease, watching for commands or changes.
pub async fn task(context: Context, control: ControlState) {
    let mut control = control;

    loop {
        tokio::select! {
            // Wait for lease control messages.
            request = control.commands.recv() => {
                let command = match request {
                    Some(command) => command,
                    // Exit the control loop and stop the task when commands can't be receive.
                    // This happens when the Lease holder is cleanly dropped.
                    None => return,
                };
                match command {
                    ControlCommands::StepDown(delay, response) => {
                        // Handle step down request with the backend.
                        if let Err(error) = control.lease.step_down(&context).await {
                            if response.send(Err(error)).is_err() {
                                // Exit the control loop when response can't be sent.
                                // This happens when the Lease holder is cleanly dropped.
                                return;
                            }
                            continue;
                        }

                        // Configure the re-election delay.
                        let delay = tokio::time::sleep(delay);
                        control.candidate = false;
                        control.step_down_delay = Box::pin(delay);
                        control.step_down_delay_used = true;
                        if response.send(Ok(())).is_err() {
                            // Exit the control loop when response can't be sent.
                            // This happens when the Lease holder is cleanly dropped.
                            return;
                        }
                    }
                };
            }

            // Wait for lease change notifications.
            state = control.lease.watch(&context, control.candidate) => {
                // Send the new state to the lease holder.
                if control.states.send(state).await.is_err() {
                    // Exit the control loop and stop the task when state changes can't be sent.
                    // This happens when the Lease holder is cleanly dropped.
                    return;
                }
            }

            // Wait for step-down time to expire and return to candidate state.
            _ = control.step_down_delay.as_mut(), if control.step_down_delay_used => {
                control.candidate = true;
                control.step_down_delay_used = false;
            }
        }
    }
}
