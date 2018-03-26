//! HTTP API interface to interact with replicante.
//!
//! This interface is a wrapper around the [`iron`] framework.
//! This module does not implement all of the APIs but rather provides
//! tools for other interfaces and components to add their own endpoints.
use std::thread;
use std::thread::JoinHandle;

use iron::Iron;
use slog::Logger;

use replicante_util_iron::MetricsHandler;

use super::super::Result;
use super::metrics::Metrics;


mod config;
mod middleware;
mod router;
mod routes;

pub use self::config::Config;
use self::middleware::RequestLogger;
use self::router::RouterBuilder;


/// The replicante HTTP API interface.
pub struct API {
    config: Config,
    handle: Option<JoinHandle<()>>,
    logger: Logger,
    router: Option<RouterBuilder>,
}

impl API {
    /// Creates a new API interface.
    pub fn new(config: Config, logger: Logger, metrics: &Metrics) -> API {
        let registry = metrics.registry().clone();
        let mut router = RouterBuilder::new();
        router.get("/", routes::root_index, "index");
        router.get("/api/v1/metrics", MetricsHandler::new(registry), "metrics");

        API {
            config,
            handle: None,
            logger,
            router: Some(router),
        }
    }

    /// Creates an Iron server and spawns a thread to serve it.
    pub fn run(&mut self) -> Result<()> {
        let bind = self.config.bind.clone();
        let logger = self.logger.clone();

        let mut chain = self.router.take().unwrap().build();
        chain.link_after(RequestLogger::new(self.logger.clone()));

        self.handle = Some(thread::spawn(move || {
            info!(logger, "Starting API server"; "bind" => bind.clone());
            Iron::new(chain).http(bind).expect("Unable to start API server");
        }));
        Ok(())
    }

    /// Wait for the server thread to stop.
    pub fn wait(&mut self) -> Result<()> {
        info!(self.logger, "Waiting for API server to stop");
        self.handle.take().map(|handle| handle.join());
        Ok(())
    }
}
