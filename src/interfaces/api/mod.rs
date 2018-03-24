//! HTTP API interface to interact with replicante.
//!
//! This interface is a wrapper around the [`iron`] framework.
//! This module does not implement all of the APIs but rather provides
//! tools for other interfaces and components to add their own endpoints.
use std::thread;
use std::thread::JoinHandle;

use iron::Iron;
use slog::Logger;

use super::super::Result;


mod router;
mod routes;

use self::router::RouterBuilder;


/// The replicante HTTP API interface.
pub struct API {
    handle: Option<JoinHandle<()>>,
    logger: Logger,
    router: Option<RouterBuilder>,
}

impl API {
    /// Creates a new API interface.
    pub fn new(logger: Logger) -> API {
        let mut router = RouterBuilder::new();
        router.get("/", routes::root_index, "index");
        API {
            handle: None,
            logger,
            router: Some(router),
        }
    }

    /// Creates an Iron server and spawns a thread to serve it.
    pub fn run(&mut self) -> Result<()> {
        let bind = String::from("127.0.0.1:16016");
        let chain = self.router.take().unwrap().build();
        let logger = self.logger.clone();
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
