//! HTTP API interface to interact with replicante.
//!
//! This interface is a wrapper around the [`iron`] framework.
//! This module does not implement all of the APIs but rather provides
//! tools for other interfaces and components to add their own endpoints.
use std::thread;
use std::thread::JoinHandle;

use iron::Chain;
use iron::Handler;
use iron::Iron;
use iron::IronResult;
use iron::Request;
use iron::Response;

use iron::method;
use iron::status;
use router::Router;

use slog::Logger;

use super::super::Result;


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
        router.get("/", root_index, "index");
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


/// Root index (`/`) handler.
fn root_index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Replicante API server")))
}


/// A builder object for an `iron-router` [`Router`].
///
/// [`Router`]: router/struct.Router.html
pub struct RouterBuilder {
    router: Router,
}

impl RouterBuilder {
    /// Create a new [`Router`] builder.
    ///
    /// [`Router`]: router/struct.Router.html
    pub fn new() -> RouterBuilder {
        let router = Router::new();
        RouterBuilder { router }
    }

    /// Converts this builder into an iron [`Chain`].
    ///
    /// [`Chain`]: iron/middleware/struct.Chain.html
    pub fn build(self) -> Chain {
        Chain::new(self.router)
    }


    /// Wrapper for [`Router::route`].
    ///
    /// [`Router::route`]: router/struct.Router.html#method.route
    pub fn route<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self, method: method::Method, glob: S, handler: H, route_id: I
    ) -> &mut RouterBuilder {
        self.router.route(method, glob, handler, route_id);
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self, glob: S, handler: H, route_id: I
    ) -> &mut RouterBuilder {
        self.route(method::Get, glob, handler, route_id)
    }
}
