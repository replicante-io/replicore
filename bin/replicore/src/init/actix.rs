//! Generic ActixWeb Server initialisation logic.
use actix_web::web::ServiceConfig;
use actix_web::HttpServer;
use anyhow::Result;

use replisdk::runtime::actix_web::AppConfigurer;
use replisdk::runtime::actix_web::AppFactory;
use replisdk::runtime::actix_web::ServerConfig;

use replicore_auth::access::Authoriser;
use replicore_auth::identity::Authenticator;
use replicore_context::Context;

use crate::api::context::ContextMiddleware;

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
    pub fn run(self, args: ActixServerRunArgs) -> Result<actix_web::dev::Server> {
        // Prepare all components needed to run the server.
        let context_middleware =
            ContextMiddleware::new(args.context, args.authenticator, args.authoriser);
        let factory = AppFactory::configure(self.app, self.conf.clone())
            .metrics(REQUEST_METRICS_PREFIX, self.metrics)
            .done();

        // Initialise and run actix server.
        let server = HttpServer::new(move || {
            let app = factory.initialise().wrap(context_middleware.clone());
            factory.finalise(app)
        })
        .disable_signals();
        let server = self.conf.apply(server)?;
        Ok(server.run())
    }

    /// Add a server configuration closure to be applied when the server is started.
    pub fn with_config<F>(&mut self, config: F) -> &mut Self
    where
        F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    {
        self.app.with_config(config);
        self
    }
}

/// Collection of server runtime configuration arguments.
pub struct ActixServerRunArgs {
    /// Interface to the requests authentication service.
    pub authenticator: Authenticator,

    /// Interface to the requests authorisation service.
    pub authoriser: Authoriser,

    /// Top-level context the server will use to derive request contexts..
    pub context: Context,
}
