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


#[cfg(test)]
mod tests {
    use iron::Headers;
    use iron::IronResult;
    use iron::Request;
    use iron::Response;

    use iron::method;
    use iron::status;

    use iron_test::request;
    use iron_test::response;

    use super::RouterBuilder;


    fn mock_get(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "GET")))
    }

    fn mock_put(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "PUT")))
    }

    #[test]
    fn attach_get() {
        let mut builder = RouterBuilder::new();
        builder.get("/", &mock_get, "test");
        let router = builder.build();

        let response = request::get("http://host:16016/", Headers::new(), &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
    }

    #[test]
    fn attach_route() {
        let mut builder = RouterBuilder::new();
        builder.route(method::Put, "/", &mock_put, "test");
        let router = builder.build();

        let response = request::put("http://host:16016/", Headers::new(), "", &router).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "PUT");
    }
}
