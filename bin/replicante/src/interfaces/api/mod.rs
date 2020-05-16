//! HTTP API interface to interact with replicante.
//!
//! This interface is a wrapper around the [`iron`] framework.
//! This module does not implement all of the APIs but rather provides
//! tools for other interfaces and components to add their own endpoints.
use actix_web::middleware;
use actix_web::App;
use actix_web::HttpServer;
use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use openssl::ssl::SslAcceptor;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use openssl::ssl::SslVerifyMode;
use slog::info;
use slog::Logger;

#[cfg(test)]
use replicante_service_coordinator::mock::MockCoordinator;
use replicante_service_coordinator::Coordinator;
use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::AppConfig;
use replicante_util_actixweb::LoggingMiddleware;
use replicante_util_actixweb::MetricsMiddleware;
use replicante_util_actixweb::SentryMiddleware;
use replicante_util_upkeep::Upkeep;

use super::healthchecks::HealthResultsCache;
use super::metrics::Metrics;
use crate::config::SentryCaptureApi;
use crate::Config as FullConfig;
use crate::ErrorKind;
use crate::Result;

mod config;
mod metrics;
mod roots;
mod routes;

pub use self::config::Config;
pub use self::metrics::register_metrics;
pub use self::roots::APIRoot;

use self::metrics::REQUESTS;

/// Context for `AppConfig` configuration callbacks.
pub type AppConfigContext<'a> = replicante_util_actixweb::AppConfigContext<'a, APIContext>;

/// The replicante HTTP API interface.
pub struct API {
    config: FullConfig,
    later: Option<LateConfig>,
    logger: Logger,
}

impl API {
    /// Creates a new API interface.
    pub fn new(
        config: FullConfig,
        coordinator: Coordinator,
        logger: Logger,
        metrics: &Metrics,
        healthchecks: HealthResultsCache,
    ) -> API {
        let later = LateConfig {
            app_config: AppConfig::default(),
            coordinator,
            health: healthchecks,
            registry: metrics.registry().clone(),
        };
        API {
            config,
            later: Some(later),
            logger,
        }
    }

    /// Register an app configuration function to be run later.
    pub fn configure<F>(&mut self, config: F)
    where
        F: Fn(&mut AppConfigContext) + 'static + Send + Sync,
    {
        self.later
            .as_mut()
            .expect("API configuration must be done before API::run is called")
            .app_config
            .register(config);
    }

    /// Creates an Iron server and spawns a thread to serve it.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let later = self.later.take().expect("LateConfig not available to take");
        let config = self.config.clone();
        let logger = self.logger.clone();
        let sentry_capture_api = config
            .sentry
            .as_ref()
            .map(|sentry| sentry.capture_api_errors.clone())
            .unwrap_or_default();

        // Extend the API server configuration with routes from `self::routes`.
        // Only `app_config` will then move into the closure, with all the dependencies
        // tucked away into the `AppConfig::register`ed closures.
        let app_config = {
            let coordinator = later.coordinator;
            let health = later.health;
            let registry = later.registry;
            let configure = self::routes::configure(health, coordinator, registry);
            let mut app_config = later.app_config;
            app_config.register(configure);
            app_config
        };

        let (send_server, receive_server) = std::sync::mpsc::sync_channel(0);
        let handle = ThreadBuilder::new("r:i:api")
            .full_name("replicore:interface:api")
            .spawn(move |scope| {
                let api = config.api.clone();
                let init_logger = logger.clone();
                let api_context = APIContext {
                    flags: config.api.trees.clone().into(),
                    config,
                };
                let mut server = HttpServer::new(move || {
                    // Register application middlewares.
                    // Remember that middlewares are executed in reverse registration order.
                    let app = App::new()
                        .wrap(LoggingMiddleware::new(logger.clone()))
                        .wrap(MetricsMiddleware::new(REQUESTS.clone()))
                        .wrap(middleware::Compress::default());
                    // Add the sentry middleware if configured.
                    let app = match sentry_capture_api {
                        SentryCaptureApi::Client => app.wrap(SentryMiddleware::new(400)),
                        SentryCaptureApi::Server => app.wrap(SentryMiddleware::new(500)),
                        // acitx-web is so type safe that apps wrapped in middlewares change type.
                        // This means that even if we do not want to use the sentry middleware we need
                        // to configure it or we can't return a consisten type from this match.
                        SentryCaptureApi::No => app.wrap(SentryMiddleware::new(600)),
                    };

                    // Configure and return the ActixWeb App
                    let mut app_config = app_config.clone();
                    app.configure(|app| app_config.configure(app, &api_context))
                })
                .keep_alive(api.timeouts.keep_alive);
                if let Some(read) = api.timeouts.read {
                    server = server.client_timeout(read * 1000);
                }
                if let Some(write) = api.timeouts.write {
                    server = server.client_shutdown(write * 1000);
                }
                if let Some(threads_count) = api.threads_count {
                    server = server.workers(threads_count);
                }

                // Configure TLS/HTTPS if enabled and bind to the given address.
                let server = match api.tls {
                    None => server.bind(&api.bind).expect("unable to bind API server"),
                    Some(tls) => {
                        let mut builder = SslAcceptor::mozilla_modern(SslMethod::tls())
                            .expect("unable to initialse TLS acceptor for API server");
                        builder
                            .set_certificate_file(&tls.server_cert, SslFiletype::PEM)
                            .expect("unable to set TLS server public certificate");
                        builder
                            .set_private_key_file(&tls.server_key, SslFiletype::PEM)
                            .expect("unable to set TLS server privte key");
                        if let Some(bundle) = tls.clients_ca_bundle {
                            builder
                                .set_ca_file(&bundle)
                                .expect("unable to set clients CAs bundle");
                            builder.set_verify(
                                SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT,
                            );
                        }
                        server
                            .bind_openssl(&api.bind, builder)
                            .expect("unable to bind API server")
                    }
                };

                // Start HTTP server and block until shutdown.
                info!(init_logger, "Starting API server"; "bind" => &api.bind);
                scope.activity("running https://actix.rs/ HTTP(S) server");
                let mut runner = actix_rt::System::new("replicore:interface:api");
                let server = server.run();
                send_server
                    .send(server.clone())
                    .expect("unable to send back server handle");
                runner.block_on(server).expect("unable to run API server");
            })
            .with_context(|_| ErrorKind::ThreadSpawn("http server"))?;
        upkeep.register_thread(handle);
        let server = receive_server
            .recv()
            .with_context(|_| ErrorKind::InterfaceInit("api"))?;
        upkeep.on_shutdown(move || {
            futures::executor::block_on(server.stop(true));
        });
        Ok(())
    }

    /// Returns an `API` instance usable as a mock.
    #[cfg(test)]
    pub fn mock(
        logger: Logger,
        metrics: &Metrics,
        healthchecks: HealthResultsCache,
    ) -> (API, MockCoordinator) {
        let config = FullConfig::mock();
        let coordinator = MockCoordinator::new(logger.clone());
        let api = API::new(config, coordinator.mock(), logger, metrics, healthchecks);
        (api, coordinator)
    }
}

/// Context for `AppConfig` configuration callbacks.
#[derive(Clone)]
pub struct APIContext {
    pub config: FullConfig,
    pub flags: APIFlags,
}

/// Container for interfaces and data used by the API server thread.
#[derive(Clone)]
struct LateConfig {
    app_config: AppConfig<APIContext>,
    coordinator: Coordinator,
    health: HealthResultsCache,
    registry: prometheus::Registry,
}
