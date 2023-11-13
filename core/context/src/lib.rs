//! The [`Context`] is a general purpose immutable container to carry scoped values around.
//!
//! Code executed as part of RepliCore processes can access operation scoped values.
//!
//! Contexts are organised into a tree structure:
//!
//! - A root context represents the general process wide scope.
//! - Derived contexts represents a narrower scope within their parent with additional
//!   or updated information attached to them.
//!
//! For example: [`Context`]s provide access to the current [`Logger`].
//! For the root context this is the process-wide logger with no additional attributes.
//! But for individual operations a derived context can be provided with a [`Logger`] decorated
//! with the operation trace ID or other request attributes.
use std::future::Ready;

use actix_web::dev::Payload;
use actix_web::Error;
use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::trace::TraceId;
use opentelemetry_api::Context as OtelContext;
use slog::Logger;
use slog::OwnedKV;
use slog::SendSyncRefUnwindSafeKV;

use replisdk::core::models::auth::Action;
use replisdk::core::models::auth::AuthContext;
use replisdk::core::models::auth::Resource;

/// The [`Context`] is a general purpose container to carry scoped values around.
///
/// Refer to the [crate level docs](crate) for details.
#[derive(Clone, Debug)]
pub struct Context {
    /// Result of the authentication process for the current request.
    ///
    /// The initial value of `None` indicates no authentication process was performed on.
    pub auth: Option<AuthContext>,

    /// Logger with contextual attributes attached to it.
    pub logger: Logger,
}

impl Context {
    /// Derive a new [`Context`] by making changes to the current one.
    pub fn derive(&self) -> ContextBuilder {
        ContextBuilder {
            auth: self.auth.clone(),
            logger: self.logger.clone(),
        }
    }

    /// Derive a new [`Context`] by making changes to the current one using the provided callback.
    pub fn derive_with<F>(&self, callback: F) -> Context
    where
        F: FnOnce(ContextBuilder) -> ContextBuilder,
    {
        let builder = callback(self.derive());
        builder.build()
    }

    /// Initialise a new root context with no values attached.
    pub fn root(logger: Logger) -> ContextBuilder {
        ContextBuilder { auth: None, logger }
    }
}

impl FromRequest for Context {
    type Error = Error;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let context = request
            .extensions()
            .get::<Context>()
            .expect("request has no context to extract")
            .clone();
        std::future::ready(Ok(context))
    }
}

/// A builder for root and derived contexts.
pub struct ContextBuilder {
    auth: Option<AuthContext>,
    logger: Logger,
}

impl ContextBuilder {
    /// Mark the context to be created as authenticated by as specified.
    pub fn authenticated(mut self, auth: AuthContext) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Update the authentication information with a new action.
    ///
    /// This is helpful when processing the request requires the multiple actions or sub-actions
    /// which all need to be authorised during processing.
    ///
    /// ## Panics
    ///
    /// This method panics if the context was not [`authenticated`](ContextBuilder::authenticated).
    /// Doing so ensures that attempts to change the action are not ignored by
    /// incorrect ordering of operations.
    pub fn authenticated_action(mut self, action: Action) -> Self {
        if self.auth.is_none() {
            panic!(
                "ContextBuilder::authenticated_action called before ContextBuilder::authenticated"
            )
        }
        if let Some(auth) = self.auth.as_mut() {
            auth.action = action;
        }
        self
    }

    /// Update the authentication information with a new resource.
    ///
    /// This is helpful when processing the request requires the multiple resources or
    /// sub-resources which all need to be authorised during processing.
    ///
    /// ## Panics
    ///
    /// This method panics if the context was not [`authenticated`](ContextBuilder::authenticated).
    /// Doing so ensures that attempts to change the resource are not ignored by
    /// incorrect ordering of operations.
    pub fn authenticated_resource(mut self, resource: Resource) -> Self {
        if self.auth.is_none() {
            panic!(
                "ContextBuilder::authenticated_resource called before ContextBuilder::authenticated"
            )
        }
        if let Some(auth) = self.auth.as_mut() {
            auth.resource = resource;
        }
        self
    }

    /// Finalise the build process and return a new [`Context`].
    pub fn build(self) -> Context {
        Context {
            auth: self.auth,
            logger: self.logger,
        }
    }

    /// Decorate the [`Context`]'s logger with the trace ID of the current OpenTelemetry span.
    ///
    /// [`Context`]: super::Context
    pub fn log_trace(self) -> Self {
        let context = OtelContext::current();
        let span = context.span();
        let trace_id = span.span_context().trace_id();
        if trace_id == TraceId::INVALID {
            self
        } else {
            let trace_id = trace_id.to_string();
            self.log_values(slog::o!("trace_id" => trace_id))
        }
    }

    /// Update the [`Context`] logger to attach new log key/pair values.
    pub fn log_values<T>(mut self, entries: OwnedKV<T>) -> Self
    where
        T: SendSyncRefUnwindSafeKV + 'static,
    {
        self.logger = self.logger.new(entries);
        self
    }
}

#[cfg(any(test, feature = "test-fixture"))]
impl Context {
    /// Create an empty context useful for test.
    pub fn fixture() -> Context {
        let logger = Logger::root(slog::Discard, slog::o!());
        Context { auth: None, logger }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use actix_web::FromRequest;
    use actix_web::HttpMessage;

    use replisdk::core::models::auth::Action;
    use replisdk::core::models::auth::AuthContext;
    use replisdk::core::models::auth::Entity;
    use replisdk::core::models::auth::Resource;

    use super::Context;

    fn fixture_auth() -> AuthContext {
        AuthContext {
            action: Action::define("test", "derive"),
            entity: Entity::Anonymous,
            impersonate: None,
            resource: Resource {
                kind: "test/resource".to_string(),
                metadata: Default::default(),
                resource_id: "rid".to_string(),
            },
        }
    }

    #[test]
    fn derive_authenticated() {
        let root = Context::fixture();
        let auth = fixture_auth();
        let context = root.derive().authenticated(auth.clone()).build();
        assert_eq!(context.auth, Some(auth));
    }

    #[test]
    fn derive_authenticated_action() {
        let root = Context::fixture();
        let auth = fixture_auth();
        let context = root
            .derive()
            .authenticated(auth.clone())
            .authenticated_action(Action::define("test", "override"))
            .build();
        let auth = context.auth.unwrap();
        assert_eq!(auth.action.as_ref(), "test:override");
    }

    #[test]
    fn derive_authenticated_resource() {
        let root = Context::fixture();
        let auth = fixture_auth();
        let context = root
            .derive()
            .authenticated(auth.clone())
            .authenticated_resource(Resource {
                kind: "test/resource".to_string(),
                metadata: Default::default(),
                resource_id: "override".to_string(),
            })
            .build();
        let auth = context.auth.unwrap();
        assert_eq!(auth.resource.resource_id, "override");
    }

    #[test]
    fn derive_log_attributes() {
        let root = Context::fixture();
        let parent = root
            .derive()
            .log_values(slog::o!("root" => "value", "test" => "root"))
            .build();
        let context = parent
            .derive()
            .log_values(slog::o!("test" => "override"))
            .build();
        assert_eq!(format!("{:?}", context.logger.list()), "(test, test, root)");
    }

    #[test]
    fn derive_noop() {
        let parent = Context::fixture();
        let context = parent.derive().build();
        assert_eq!(
            format!("{:?}", parent.logger.list()),
            format!("{:?}", context.logger.list()),
        );
    }

    #[actix_web::test]
    async fn extract_context() {
        let context = Context::fixture();
        let request = TestRequest::get().to_http_request();
        request.extensions_mut().insert(context);
        Context::extract(&request).await.unwrap();
    }
}
