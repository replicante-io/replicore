//! RepliCore dependency Synchronisation (initialise or migrate state).
use anyhow::Result;

use replicore_conf::Conf;
use replicore_context::Context;
use replicore_context::ContextBuilder;
use replicore_events::emit::EventsFactory;
use replicore_events::emit::EventsFactorySyncArgs;

use super::actix::ActixServerRunArgs;
use super::backends::Backends;
use super::generic::GenericInit;

/// Process builder to initialise and run a RepliCore dependences sync process.
pub struct Sync {
    /// Root context for the process.
    context: ContextBuilder,

    /// Process initialisation logic common to all RepliCore commands.
    generic: GenericInit,
}

impl Sync {
    /// Register all supported backends for all process dependencies.
    ///
    /// Supported dependencies can be tuned at compile time using crate features.
    pub fn add_default_backends(mut self) -> Self {
        self.generic.add_default_backends();
        self
    }

    /// Build a server from the loaded configuration.
    pub async fn configure(conf: Conf) -> Result<Self> {
        let generic = GenericInit::configure(conf).await?;
        let context = Context::root(generic.telemetry.logger.clone());
        let sync = Self { context, generic };
        Ok(sync)
    }

    /// Register a new factory for an Events Platform implementation.
    ///
    /// # Panics
    ///
    /// This method panics if the identifier of the new Events Platform backend is already in use.
    pub fn events_backend<B, S>(mut self, id: S, backend: B) -> Self
    where
        B: EventsFactory + 'static,
        S: Into<String>,
    {
        self.generic.backends.register_events(id, backend);
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
}

/// Entrypoint to dependences synchronisation.
async fn synchronise_dependencies(context: &Context, args: SyncArgs) -> Result<()> {
    slog::info!(context.logger, "Synchronising dependences");
    // TODO: Synchronise election service.
    sync_events(context, &args).await?;
    // TODO: Synchronise persistence store.
    // TODO: Synchronise task submission queues.
    Ok(())
}

async fn sync_events(context: &Context, args: &SyncArgs) -> Result<()> {
    slog::debug!(context.logger, "Synchronising events backend");
    let eargs = EventsFactorySyncArgs {
        conf: &args.conf.events.options,
        context,
    };
    args.backends
        .events(&args.conf.events.backend)?
        .sync(eargs)
        .await
}
