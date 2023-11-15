//! RepliCore Control Plane Server initialisation as a builder.
use anyhow::Result;

use replicore_conf::Conf;
use replicore_context::Context;
use replicore_context::ContextBuilder;
use replicore_events::emit::EventsFactory;
use replicore_events::emit::EventsFactoryArgs;
use replicore_injector::Injector;

use super::actix::ActixServerRunArgs;
use super::backends::Backends;
use super::generic::GenericInit;

/// Process builder to initialise and run a RepliCore Control Plane instance.
pub struct Server {
    /// Root context for the process.
    context: ContextBuilder,

    /// Process initialisation logic common to all RepliCore commands.
    generic: GenericInit,
}

impl Server {
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
        let server = Self { context, generic };
        Ok(server)
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

        // Initialise dependencies and global injector.
        let injector = injector(&context, &self.generic.conf, &self.generic.backends).await?;
        Injector::set_global(injector);
        // Fetch the injector back out to ensure it is set correctly for the process.
        let injector = Injector::global();

        // Start execution of all process components.
        self.generic.run_server(
            &context,
            ActixServerRunArgs {
                authenticator: injector.authenticator,
                authoriser: injector.authoriser,
                context: injector.context,
            },
        )?;
        // TODO: Add other components

        // Run until user-requested exit or process error.
        self.generic.wait().await
    }
}

/// Initialise all backends and collected them into an [`Injector`] object.
pub async fn injector(context: &Context, conf: &Conf, backends: &Backends) -> Result<Injector> {
    // Grab all dependencies factories.
    let conf = conf.clone();
    let events = backends.events(&conf.events.backend)?;

    // Initialise all dependencies.
    let events = events
        .events(EventsFactoryArgs {
            conf: &conf.events.options,
            context,
        })
        .await?;

    // Auth* is not currently configurable and just in place for the future.
    let authenticator = replicore_auth_insecure::Anonymous.into();
    let authoriser = replicore_auth::access::Authoriser::wrap(
        replicore_auth_insecure::Unrestricted,
        events.clone(),
    );

    // Combine then into an Injector object.
    let injector = Injector {
        authenticator,
        authoriser,
        conf,
        context: context.clone(),
        events,
    };
    Ok(injector)
}
