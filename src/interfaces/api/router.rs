use iron::Chain;
use iron::Handler;
use iron::method;
use router::Router;


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
