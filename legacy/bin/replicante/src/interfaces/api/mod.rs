use std::time::Duration;

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
use replicante_util_upkeep::Upkeep;

use super::healthchecks::HealthResultsCache;
use super::metrics::Metrics;
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
pub struct Api {
    config: FullConfig,
    later: Option<LateConfig>,
    logger: Logger,
}

impl Api {
    /// Creates a new API interface.
    pub fn new(
        config: FullConfig,
        coordinator: Coordinator,
        logger: Logger,
        metrics: &Metrics,
        healthchecks: HealthResultsCache,
    ) -> Api {
        let later = LateConfig {
            app_config: AppConfig::default(),
            coordinator,
            health: healthchecks,
            registry: metrics.registry().clone(),
        };
        Api {
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
            .map(|sentry| sentry.capture_api_errors)
            .unwrap_or(true);

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
                    // Register application middleware.
                    // Remember that middleware are executed in reverse registration order.
                    let app = App::new()
                        .wrap(LoggingMiddleware::new(logger.clone()))
                        .wrap(MetricsMiddleware::new(REQUESTS.clone()))
                        .wrap(middleware::Compress::default());

                    // Add the sentry middleware if configured.
                    let sentry_capture = sentry_actix::Sentry::builder()
                        .capture_server_errors(sentry_capture_api)
                        .emit_header(true)
                        .finish();
                    let app = app.wrap(sentry_capture);

                    // If no route matches requests return a 404 with a JSON body
                    // like all APIs should do.
                    let app = app
                        .default_service(actix_web::web::route().to(routes::not_found_empty_json));

                    // Configure and return the ActixWeb App
                    let mut app_config = app_config.clone();
                    app.configure(|app| app_config.configure(app, &api_context))
                });
                if let Some(keep_alive) = api.timeouts.keep_alive {
                    let keep_alive = Duration::from_secs(keep_alive);
                    server = server.keep_alive(keep_alive);
                }
                if let Some(read) = api.timeouts.read {
                    let read = Duration::from_secs(read);
                    server = server.client_request_timeout(read);
                }
                if let Some(write) = api.timeouts.write {
                    let write = Duration::from_secs(write);
                    server = server.client_disconnect_timeout(write);
                }
                if let Some(threads_count) = api.threads_count {
                    server = server.workers(threads_count);
                }

                // Configure TLS/HTTPS if enabled and bind to the given address.
                let server = match api.tls {
                    None => server.bind(&api.bind).expect("unable to bind API server"),
                    Some(tls) => {
                        let mut builder = SslAcceptor::mozilla_modern(SslMethod::tls())
                            .expect("unable to initialise TLS acceptor for API server");
                        builder
                            .set_certificate_file(&tls.server_cert, SslFiletype::PEM)
                            .expect("unable to set TLS server public certificate");
                        builder
                            .set_private_key_file(&tls.server_key, SslFiletype::PEM)
                            .expect("unable to set TLS server private key");
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
                let runner = actix_rt::System::new();
                let server = server.run();
                send_server
                    .send(server.handle())
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
    ) -> (Api, MockCoordinator) {
        let config = FullConfig::mock();
        let coordinator = MockCoordinator::default();
        let api = Api::new(config, coordinator.mock(), logger, metrics, healthchecks);
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
