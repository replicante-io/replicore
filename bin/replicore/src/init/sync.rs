//! RepliCore dependency Synchronisation (initialise or migrate state).
use anyhow::Result;

use replicore_conf::Conf;
use replicore_context::Context;
use replicore_context::ContextBuilder;
use replicore_events::emit::EventsFactory;
use replicore_events::emit::EventsFactorySyncArgs;
use replicore_store::StoreFactory;
use replicore_store::StoreFactorySyncArgs;
use replicore_tasks::conf::Queue;
use replicore_tasks::factory::TasksFactory;
use replicore_tasks::factory::TasksFactorySyncArgs;

use super::actix::ActixServerRunArgs;
use super::backends::Backends;
use super::generic::GenericInit;

/// Process builder to initialise and run a RepliCore dependences sync process.
pub struct Sync {
    /// Root context for the process.
    context: ContextBuilder,

    /// Process initialisation logic common to all RepliCore commands.
    generic: GenericInit,

    /// All queues known to the process to be configured with the backend.
    task_queues: Vec<&'static Queue>,
}

impl Sync {
    /// Build a server from the loaded configuration.
    pub async fn configure(conf: Conf) -> Result<Self> {
        let generic = GenericInit::configure(conf).await?;
        let context = Context::root(generic.telemetry.logger.clone());
        let sync = Self {
            context,
            generic,
            task_queues: Default::default(),
        };
        Ok(sync)
    }

    /// Register all task queues required by the control plane to operate.
    pub fn register_core_tasks(mut self) -> Self {
        let queues = &mut self.task_queues;
        queues.push(&replicore_task_discovery::DISCOVERY_QUEUE);
        queues.push(&replicore_task_orchestrate::ORCHESTRATE_QUEUE);
        self
    }

    /// Register all supported backends for all process dependencies.
    ///
    /// Supported dependencies can be tuned at compile time using crate features.
    pub fn register_default_backends(mut self) -> Self {
        self.generic.register_default_backends();
        self
    }

    /// Register a new factory for an Events Platform implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Events Platform backend is already in use.
    pub fn register_events<B, S>(mut self, id: S, backend: B) -> Self
    where
        B: EventsFactory + 'static,
        S: Into<String>,
    {
        self.generic.backends.register_events(id, backend);
        self
    }

    /// Register a new factory for a Persistent Store implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Persistent Store backend is already in use.
    pub fn register_store<B, S>(mut self, id: S, backend: B) -> Self
    where
        B: StoreFactory + 'static,
        S: Into<String>,
    {
        self.generic.backends.register_store(id, backend);
        self
    }

    /// Register a new factory for a Background Tasks queue implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Background Tasks backend is already in use.
    pub fn register_tasks<B, S>(mut self, id: S, backend: B) -> Self
    where
        B: TasksFactory + 'static,
        S: Into<String>,
    {
        self.generic.backends.register_tasks(id, backend);
        self
    }

    /// Finalise process initialisation and run the RepliCore server.
    pub async fn run(mut self) -> Result<()> {
        // Prepare for late process initialisation.
        let context = self.context.build();
        self.generic
            .validate_backends_conf(&context)?
            .register_metrics()?;
        self.generic.run_server(
            &context,
            ActixServerRunArgs {
                authenticator: replicore_auth_insecure::Anonymous.into(),
                authoriser: replicore_auth::access::Authoriser::wrap(
                    replicore_auth_insecure::Unrestricted,
                    crate::backends::EventsNull.into(),
                ),
                context: context.clone(),
            },
        )?;

        // Sync dependencies in a dedicated task and auto-exit when done.
        let args = SyncArgs {
            backends: self.generic.backends.clone(),
            conf: self.generic.conf.clone(),
            queues: self.task_queues,
        };
        self.generic.shutdown.watch_tokio(tokio::spawn(async move {
            synchronise_dependencies(&context, args).await
        }));

        // Wait for the sync process to complete, the user to request an exit, or an error.
        self.generic.wait().await
    }
}

/// Arguments passed around to sync functions.
struct SyncArgs {
    backends: Backends,
    conf: Conf,
    queues: Vec<&'static Queue>,
}

/// Entrypoint to dependences synchronisation.
async fn synchronise_dependencies(context: &Context, args: SyncArgs) -> Result<()> {
    slog::info!(context.logger, "Synchronising dependences");
    // TODO: Synchronise election service.
    sync_events(context, &args).await?;
    sync_store(context, &args).await?;
    sync_tasks(context, &args).await?;
    Ok(())
}

async fn sync_events(context: &Context, args: &SyncArgs) -> Result<()> {
    slog::debug!(context.logger, "Synchronising events backend");
    let sync_args = EventsFactorySyncArgs {
        conf: &args.conf.events.options,
        context,
    };
    args.backends
        .events(&args.conf.events.backend)?
        .sync(sync_args)
        .await
}

async fn sync_store(context: &Context, args: &SyncArgs) -> Result<()> {
    slog::debug!(context.logger, "Synchronising persistent store backend");
    let sync_args = StoreFactorySyncArgs {
        conf: &args.conf.store.options,
        context,
    };
    args.backends
        .store(&args.conf.store.backend)?
        .sync(sync_args)
        .await
}

async fn sync_tasks(context: &Context, args: &SyncArgs) -> Result<()> {
    slog::debug!(context.logger, "Synchronising background tasks backend");
    let sync_args = TasksFactorySyncArgs {
        conf: &args.conf.store.options,
        context,
        queues: &args.queues,
    };
    args.backends
        .tasks(&args.conf.tasks.service.backend)?
        .sync(sync_args)
        .await
}
