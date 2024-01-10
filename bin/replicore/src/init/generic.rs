//! Process initialisation builder for aspects to initialise for all commands.
use std::time::Duration;

use anyhow::Result;

use replisdk::runtime::shutdown::ShutdownManager;
use replisdk::runtime::shutdown::ShutdownManagerBuilder;
use replisdk::runtime::telemetry;
use replisdk::runtime::telemetry::Telemetry;
use replisdk::runtime::telemetry::TelemetryConfig;
use replisdk::runtime::telemetry::TelemetryOptions;

use replicore_conf::Conf;
use replicore_context::Context;

use super::actix::ActixServer;
use super::actix::ActixServerRunArgs;
use super::backends::Backends;

/// Process builder to initialise all RepliCore commands.
pub struct GenericInit {
    pub api: ActixServer,
    pub backends: Backends,
    pub conf: Conf,
    pub shutdown: ShutdownManagerBuilder<()>,
    pub telemetry: Telemetry,
}

impl GenericInit {
    /// Build a server from the loaded configuration.
    pub async fn configure(conf: Conf) -> Result<Self> {
        let telemetry = telemetry(conf.telemetry.clone()).await?;
        let api = ActixServer::new(conf.http.clone(), telemetry.metrics.clone());
        let shutdown = shutdown_manager(telemetry.logger.clone(), &conf);
        let server = Self {
            api,
            backends: Default::default(),
            conf,
            shutdown,
            telemetry,
        };
        Ok(server)
    }

    /// Register all supported backends for all process dependencies.
    ///
    /// Supported dependencies can be tuned at compile time using crate features.
    pub fn register_default_backends(&mut self) -> &mut Self {
        #[cfg(feature = "replicore-events-sqlite")]
        self.backends
            .register_events("sqlite", replicore_events_sqlite::emit::SQLiteFactory)
            .register_store("sqlite", replicore_store_sqlite::SQLiteFactory)
            .register_tasks("sqlite", replicore_tasks_sqlite::SQLiteFactory);
        self
    }

    /// Register metrics for all selected backends.
    pub fn register_metrics(&self) -> Result<&Self> {
        self.backends
            .events(&self.conf.events.backend)?
            .register_metrics(&self.telemetry.metrics)?;
        self.backends
            .store(&self.conf.store.backend)?
            .register_metrics(&self.telemetry.metrics)?;
        self.backends
            .tasks(&self.conf.tasks.service.backend)?
            .register_metrics(&self.telemetry.metrics)?;
        Ok(self)
    }

    // Configure and run the API server.
    pub fn run_server(
        &mut self,
        context: &Context,
        server_args: ActixServerRunArgs,
    ) -> Result<&mut Self> {
        slog::debug!(context.logger, "Starting API server");
        let server = self.api.clone().run(server_args)?;
        self.shutdown.watch_actix(server, ());
        slog::info!(
            context.logger, "API server listening for connection";
            "address" => &self.conf.http.bind,
        );
        Ok(self)
    }

    /// Validate the loaded configuration objects for the selected backends.
    pub fn validate_backends_conf(&self, context: &Context) -> Result<&Self> {
        self.backends
            .events(&self.conf.events.backend)?
            .conf_check(context, &self.conf.events.options)?;
        self.backends
            .store(&self.conf.store.backend)?
            .conf_check(context, &self.conf.store.options)?;
        self.backends
            .tasks(&self.conf.tasks.service.backend)?
            .conf_check(context, &self.conf.tasks.service.options)?;
        Ok(self)
    }

    /// Initialisation done, wait until the process fails or the user shuts it down.
    pub async fn wait(self) -> Result<()> {
        slog::info!(
            self.telemetry.logger,
            "RepliCore process initialisation complete"
        );
        let exit = self.shutdown.build();
        exit.wait().await
    }
}

/// Initialise process shutdown manager.
pub fn shutdown_manager(logger: slog::Logger, conf: &Conf) -> ShutdownManagerBuilder<()> {
    let grace = Duration::from_secs(conf.runtime.shutdown_grace_sec);
    let mut shutdown = ShutdownManager::builder();
    shutdown
        .logger(logger)
        .graceful_shutdown_timeout(grace)
        .watch_signal_with_default();
    shutdown
}

/// Initialise process telemetry.
pub async fn telemetry(conf: TelemetryConfig) -> Result<Telemetry> {
    let telemetry_options = TelemetryOptions::for_sentry_release(super::RELEASE_ID)
        .for_app(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .finish();
    let telemetry = telemetry::initialise(conf, telemetry_options).await?;
    slog::info!(telemetry.logger, "Process telemetry initialised");
    Ok(telemetry)
}
