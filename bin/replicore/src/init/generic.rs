//! Process initialisation builder for aspects to initialise for all commands.
use std::time::Duration;

//use actix_web::web::ServiceConfig;
use actix_web::HttpServer;
use anyhow::Result;

use replisdk::runtime::actix_web::AppConfigurer;
use replisdk::runtime::actix_web::AppFactory;
use replisdk::runtime::actix_web::ServerConfig;
use replisdk::runtime::shutdown::ShutdownManager;
use replisdk::runtime::shutdown::ShutdownManagerBuilder;
use replisdk::runtime::telemetry;
use replisdk::runtime::telemetry::Telemetry;
use replisdk::runtime::telemetry::TelemetryConfig;
use replisdk::runtime::telemetry::TelemetryOptions;

use replicore_conf::Conf;

/// Prefix for request metrics names.
const REQUEST_METRICS_PREFIX: &str = "replicore";

/// Builder pattern to configure and start an ActixWeb Server.
#[derive(Clone)]
pub struct ActixServer {
    app: AppConfigurer,
    conf: ServerConfig,
    metrics: prometheus::Registry,
}

impl ActixServer {
    /// Create an ActixWeb Server configuration builder.
    pub fn new(conf: ServerConfig, metrics: prometheus::Registry) -> Self {
        ActixServer {
            app: Default::default(),
            conf,
            metrics,
        }
    }

    /// Convert the builder into an [`HttpServer`](actix_web::HttpServer) and run it.
    pub fn run(self) -> Result<actix_web::dev::Server> {
        let factory = AppFactory::configure(self.app, self.conf.clone())
            .metrics(REQUEST_METRICS_PREFIX, self.metrics)
            .done();
        let server = HttpServer::new(move || {
            let app = factory.initialise();
            // TODO: context configuration once replicore has it.
            factory.finalise(app)
        });
        let server = self.conf.apply(server)?;
        Ok(server.run())
    }

    ///// Add a server configuration closure to be applied when the server is started.
    //pub fn with_config<F>(&mut self, config: F) -> &mut Self
    //where
    //    F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    //{
    //    self.app.with_config(config);
    //    self
    //}
}

/// Process builder to initialise all RepliCore commands.
pub struct GenericInit {
    pub api: ActixServer,
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
            conf,
            shutdown,
            telemetry,
        };
        Ok(server)
    }

    // Configure and run the API server.
    pub fn run_server(mut self) -> Result<Self> {
        slog::debug!(self.telemetry.logger, "Starting API server");
        let server = self.api.clone().run()?;
        self.shutdown = self.shutdown.watch_actix(server, ());
        slog::info!(
            self.telemetry.logger, "API server listening for connection";
            "address" => &self.conf.http.bind,
        );
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
    ShutdownManager::builder()
        .logger(logger)
        .graceful_shutdown_timeout(grace)
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
