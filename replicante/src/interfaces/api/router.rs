use iron::Chain;
use iron::Handler;
use iron::method;
use router::Router;

use super::config::APIVersions;
use super::APIVersion;

/// A builder object for an `iron-router` [`Router`].
///
/// [`Router`]: router/struct.Router.html
pub struct RouterBuilder {
    config: APIVersions,
    router: Router,
}

impl RouterBuilder {
    /// Create a new [`Router`] builder.
    ///
    /// [`Router`]: router/struct.Router.html
    pub fn new(config: APIVersions) -> RouterBuilder {
        let router = Router::new();
        RouterBuilder { config, router }
    }

    /// Convert this builder into an iron [`Chain`].
    ///
    /// [`Chain`]: iron/middleware/struct.Chain.html
    pub fn build(self) -> Chain {
        Chain::new(self.router)
    }

    /// Register routes for a specific API version.
    pub fn for_version(&mut self, version: APIVersion) -> VersionedRouter {
        let prefix = version.prefix();
        let enabled = match version {
            APIVersion::Unstable => self.config.unstable,
        };
        let router = &mut self.router;
        VersionedRouter { enabled, prefix, router }
    }
}

/// Specialised router to mount endpoints for a specified version.
pub struct VersionedRouter<'a> {
    enabled: bool,
    prefix: &'static str,
    router: &'a mut Router,
}

impl<'a> VersionedRouter<'a> {
    /// Like route, but specialized to the `Get` method.
    pub fn get<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut VersionedRouter<'a> {
        self.route(method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut VersionedRouter<'a> {
        self.route(method::Post, glob, handler, route_id)
    }

    /// Wrapper for [`Router::route`].
    ///
    /// [`Router::route`]: router/struct.Router.html#method.route
    pub fn route<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        method: method::Method,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut VersionedRouter<'a> {
        if !self.enabled {
            return self;
        }
        let glob = self.prefix.to_string() + glob.as_ref();
        let route_id = self.prefix.to_string() + route_id.as_ref();
        self.router.route(method, glob, handler, route_id);
        self
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

    use super::APIVersion;
    use super::APIVersions;
    use super::RouterBuilder;


    fn mock_get(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "GET")))
    }

    fn mock_put(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "PUT")))
    }

    #[test]
    fn attach_get() {
        let mut builder = RouterBuilder::new(APIVersions::default());
        {
            let mut version = builder.for_version(APIVersion::Unstable);
            version.get("/", &mock_get, "test");
        }
        let router = builder.build();
        let response = request::get("http://host:16016/api/unstable/", Headers::new(), &router)
            .unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "GET");
    }

    #[test]
    fn attach_route() {
        let mut builder = RouterBuilder::new(APIVersions::default());
        {
            let mut version = builder.for_version(APIVersion::Unstable);
            version.route(method::Put, "/", &mock_put, "test");
        }
        let router = builder.build();
        let response = request::put("http://host:16016/api/unstable/", Headers::new(), "", &router)
            .unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "PUT");
    }
}
