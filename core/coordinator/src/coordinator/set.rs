//! Set of [`ICoordinated`] exclusive tasks to execute in primary/secondary mode
//! depending on a coordination lease shared by the whole process.
use anyhow::Result;
use futures::stream::FuturesUnordered;
use futures::stream::TryStreamExt;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use replicore_context::Context;

use crate::ICoordinated;
use crate::LeaseConf;
use crate::LeaseFactory;
use crate::State;

/// Manages a set of [`ICoordinated`] instances to coordinate exclusive logic.
pub struct Coordinator {
    /// Configuration to create the coordination [`Lease`] with.
    lease_conf: LeaseConf,

    /// Factory to create the coordination [`Lease`] with.
    lease_factory: LeaseFactory,

    /// Sender side for checking or watching coordination lease state.
    state_sender: Sender<State>,

    /// Receiver side for checking or watching coordination lease state.
    state_watcher: Receiver<State>,

    /// Set of [`ICoordinated`] tasks to coordinate.
    tasks: Vec<Box<dyn ICoordinated>>,
}

impl Coordinator {
    /// Build a [`Coordinator`] instance.
    pub fn builder<F>(factory: F) -> CoordinatorBuilder
    where
        F: Into<LeaseFactory>,
    {
        CoordinatorBuilder::new(factory.into())
    }

    /// Inspect the coordination lease configuration.
    pub fn conf(&self) -> &LeaseConf {
        &self.lease_conf
    }

    /// Subscribe to coordinated lease state inspection and change notification.
    pub fn inspector(&self) -> Receiver<State> {
        self.state_watcher.clone()
    }

    /// Run the coordination component, including election and failover handling.
    pub async fn run(&self, context: &Context) -> Result<()> {
        let mut lease = self.lease_factory.get(context, &self.lease_conf).await?;
        let mut state_last = *self.state_watcher.borrow();

        loop {
            tokio::select! {
                // Watch the lease for state changes.
                state_new = lease.watch() => {
                    let state_new = state_new?;
                    self.state_transition(context, state_last, state_new).await?;
                    state_last = state_new;
                }

                // Drive execution of coordinated work based on the state of the lease.
                exec = self.handle_state(context, state_last) => {
                    // The running logic returns when the process should shut down.
                    return exec;
                }
            }
        }
    }

    /// Drive execution of coordinated work based on the state of the lease.
    async fn handle_state(&self, context: &Context, state: State) -> Result<()> {
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
    async fn state_transition(&self, context: &Context, from: State, to: State) -> Result<()> {
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

/// Builder pattern for [`Coordinator`] objects.
pub struct CoordinatorBuilder {
    /// Configuration to create the coordination [`Lease`] with.
    lease_conf: LeaseConf,

    /// Factory to create the coordination [`Lease`] with.
    lease_factory: LeaseFactory,

    /// Set of [`ICoordinated`] tasks to coordinate.
    tasks: Vec<Box<dyn ICoordinated>>,
}

impl CoordinatorBuilder {
    /// Complete the building process and returns a [`Coordinator`] instance.
    pub fn build(self) -> Coordinator {
        let (state_sender, state_watcher) = tokio::sync::watch::channel(State::Idle);
        Coordinator {
            lease_conf: self.lease_conf,
            lease_factory: self.lease_factory,
            state_sender,
            state_watcher,
            tasks: self.tasks,
        }
    }

    /// Set the lease configuration to use.
    pub fn lease_configuration(mut self, conf: LeaseConf) -> Self {
        self.lease_conf = conf;
        self
    }

    /// Start a [`Coordinator`] build with the given lease factory.
    pub fn new(factory: LeaseFactory) -> Self {
        let lease_conf = LeaseConf {
            id: "core.coordinator".into(),
        };
        CoordinatorBuilder {
            lease_conf,
            lease_factory: factory,
            tasks: Vec::new(),
        }
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

#[cfg(test)]
mod tests {
    use crate::CoordinatedFixture;
    use crate::CoordinatedFixtureNotification;
    use crate::Coordinator;
    use crate::FixedLeaseCallback;
    use crate::LeaseConf;
    use crate::LeaseFixture;
    use crate::State;

    /// Configuration for unit-test leases.
    fn lease_conf() -> LeaseConf {
        LeaseConf {
            id: "unit-test".into(),
        }
    }

    #[tokio::test]
    async fn init_to_primary() {
        let context = replicore_context::Context::fixture();
        let conf = lease_conf();
        let coordinated = CoordinatedFixture::default();
        let factory =
            LeaseFixture::factory((), || Box::new(FixedLeaseCallback::new(State::Primary)));

        let notifs = coordinated.notifications();
        let coordinator = Coordinator::builder(factory)
            .lease_configuration(conf)
            .task(coordinated)
            .build();
        coordinator.run(&context).await.unwrap();

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
        let conf = lease_conf();
        let coordinated = CoordinatedFixture::default();
        let factory =
            LeaseFixture::factory((), || Box::new(FixedLeaseCallback::new(State::Secondary)));

        let notifs = coordinated.notifications();
        let coordinator = Coordinator::builder(factory)
            .lease_configuration(conf)
            .task(coordinated)
            .build();
        coordinator.run(&context).await.unwrap();

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
