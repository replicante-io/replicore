//! Set of [`ICoordinated`] exclusive tasks to execute in primary/secondary mode
//! depending on a coordination lease shared by the whole process.
use anyhow::Result;
use futures::stream::FuturesUnordered;
use futures::stream::TryStreamExt;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use replisdk::runtime::shutdown::ShutdownHandle;

use replicore_context::Context;

use crate::ICoordinated;
use crate::LeaseBuilder;
use crate::LeaseHandle;
use crate::LeaseRegistry;
use crate::State;

/// Identifier of the Lease used to coordinate exclusive tasks.
const COORDINATOR_LEASE_ID: &str = "core.coordinator";

/// Manages a set of [`ICoordinated`] instances to coordinate exclusive logic.
pub struct Coordinator {
    /// Encapsulate coordination logic for ease of use in [`Coordinator::run`].
    inner: CoordinatorInner,

    /// Partially initialise [`Lease`] to ensure exclusive `Coordinator` execution.
    lease_builder: LeaseBuilder,

    /// Receiver side for checking or watching coordination lease state.
    state_watcher: Receiver<State>,
}

impl Coordinator {
    /// Build a [`Coordinator`] instance.
    pub async fn builder(
        context: &Context,
        registry: &LeaseRegistry,
    ) -> Result<CoordinatorBuilder> {
        CoordinatorBuilder::new(context, registry).await
    }

    /// Subscribe to coordinated lease state inspection and change notifications.
    pub fn inspector(&self) -> Receiver<State> {
        self.state_watcher.clone()
    }

    /// Run the coordination component, including election and failover handling.
    pub async fn run(self, context: &Context, exit: ShutdownHandle) -> Result<()> {
        let mut lease = self.lease_builder.build();
        let mut state_last = *self.state_watcher.borrow();

        // Aside from lease work, wait for an exit signal to step down cleanly.
        let exit = exit.wait();
        tokio::pin!(exit);

        loop {
            tokio::select! {
                // Watch the lease for state changes.
                state_new = lease.watch() => {
                    let state_new = state_new?;
                    self.inner.state_transition(context, state_last, state_new).await?;
                    state_last = state_new;
                }

                // Drive execution of coordinated work based on the state of the lease.
                exec = self.inner.handle_state(context, state_last) => {
                    // The running logic returns when the process should shut down.
                    return exec;
                }

                _ = &mut exit => break,
            }
        }

        // Step down and exit, the delay is irrelevant as we are exiting but required by the API.
        let delay = std::time::Duration::from_secs(30);
        slog::info!(context.logger, "Coordinator set stepping down");
        lease.step_down(delay).await?;
        Ok(())
    }
}

/// Builder pattern for [`Coordinator`] objects.
pub struct CoordinatorBuilder {
    /// Partially initialise [`Lease`] to ensure exclusive `Coordinator` execution.
    lease_builder: LeaseBuilder,

    /// Set of [`ICoordinated`] tasks to coordinate.
    tasks: Vec<Box<dyn ICoordinated>>,
}

impl CoordinatorBuilder {
    /// Complete the building process and returns a [`Coordinator`] instance.
    pub fn build(self) -> Coordinator {
        let (state_sender, state_watcher) = tokio::sync::watch::channel(State::Idle);
        Coordinator {
            inner: CoordinatorInner {
                state_sender,
                tasks: self.tasks,
            },
            lease_builder: self.lease_builder,
            state_watcher,
        }
    }

    /// Return a [`LeaseHandle`] to inspect and manage the coordinator lease.
    pub fn handle(&self) -> LeaseHandle {
        self.lease_builder.handle()
    }

    /// Returns `true` if no task is registered with the coordinator.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Start a [`Coordinator`] build with the given lease factory.
    pub async fn new(context: &Context, registry: &LeaseRegistry) -> Result<Self> {
        let lease_builder = registry
            .lease_builder(context, COORDINATOR_LEASE_ID)
            .await?;
        let builder = CoordinatorBuilder {
            lease_builder,
            tasks: Vec::new(),
        };
        Ok(builder)
    }

    /// Add a coordinated task to the set.
    pub fn task<T>(mut self, task: T) -> Self
    where
        T: ICoordinated + 'static,
    {
        let task = Box::new(task);
        self.tasks.push(task);
        self
    }
}

/// Encapsulate coordination logic for ease of use in [`Coordinator::run`].
struct CoordinatorInner {
    /// Sender side for checking or watching coordination lease state.
    state_sender: Sender<State>,

    /// Set of [`ICoordinated`] tasks to coordinate.
    tasks: Vec<Box<dyn ICoordinated>>,
}

impl CoordinatorInner {
    /// Drive execution of coordinated work based on the state of the lease.
    pub async fn handle_state(&self, context: &Context, state: State) -> Result<()> {
        match state {
            State::Candidate | State::Idle | State::Lost => std::future::pending().await,
            State::Primary => {
                // Await the primary handler for all tasks until one errors or ends.
                let mut tasks: FuturesUnordered<_> = self
                    .tasks
                    .iter()
                    .map(|task| task.primary(context))
                    .collect();
                tasks.try_next().await?;
                Ok(())
            }
            State::Secondary => {
                // Await the secondary handler for all tasks until one errors or ends.
                let mut tasks: FuturesUnordered<_> = self
                    .tasks
                    .iter()
                    .map(|task| task.secondary(context))
                    .collect();
                tasks.try_next().await?;
                Ok(())
            }
        }
    }

    /// Notify coordinated objects about a state transition.
    pub async fn state_transition(&self, context: &Context, from: State, to: State) -> Result<()> {
        slog::info!(
            context.logger, "Exclusive coordinator lease state changed";
            "from" => %from, "to" => %to,
        );
        self.state_sender.send(to)?;
        self.tasks
            .iter()
            .map(|task| task.transition(context, from, to))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use replisdk::runtime::shutdown::ShutdownHandle;

    use replicore_context::Context;

    use crate::CoordinatedFixture;
    use crate::CoordinatedFixtureNotification;
    use crate::Coordinator;
    use crate::FixedLeaseCallback;
    use crate::LeaseFixture;
    use crate::LeaseRegistry;
    use crate::State;

    async fn create_coordinator(
        context: &Context,
        state: State,
        coordinated: CoordinatedFixture,
    ) -> Coordinator {
        let factory = LeaseRegistry::from(LeaseFixture::factory((), move || {
            Box::new(FixedLeaseCallback::new(state))
        }));
        Coordinator::builder(&context, &factory)
            .await
            .unwrap()
            .task(coordinated)
            .build()
    }

    #[tokio::test]
    async fn init_to_primary() {
        let context = replicore_context::Context::fixture();
        let coordinated = CoordinatedFixture::default();
        let (exit, _signal) = ShutdownHandle::fixture();

        let notifs = coordinated.notifications();
        let coordinator = create_coordinator(&context, State::Primary, coordinated).await;
        coordinator.run(&context, exit).await.unwrap();

        let notifs = notifs.snapshot();
        assert_eq!(
            notifs,
            vec![
                CoordinatedFixtureNotification::Transition(State::Idle, State::Primary),
                CoordinatedFixtureNotification::Primary,
            ]
        )
    }

    #[tokio::test]
    async fn init_to_secondary() {
        let context = replicore_context::Context::fixture();
        let coordinated = CoordinatedFixture::default();
        let (exit, _signal) = ShutdownHandle::fixture();

        let notifs = coordinated.notifications();
        let coordinator = create_coordinator(&context, State::Secondary, coordinated).await;
        coordinator.run(&context, exit).await.unwrap();

        let notifs = notifs.snapshot();
        assert_eq!(
            notifs,
            vec![
                CoordinatedFixtureNotification::Transition(State::Idle, State::Secondary),
                CoordinatedFixtureNotification::Secondary,
            ]
        )
    }
}
