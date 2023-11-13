//! ActixWeb Middleware to attach [`Context`] objects to requests.
use std::collections::BTreeMap;
use std::future::Ready;
use std::sync::Arc;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpMessage;
use anyhow::Result;
use futures_util::future::LocalBoxFuture;

use replisdk::core::models::auth::Action;
use replisdk::core::models::auth::AuthContext;
use replisdk::core::models::auth::Resource;

use replicore_auth::access::Authoriser;
use replicore_auth::identity::Authenticator;
use replicore_context::Context;
use replicore_context::ContextBuilder;

/// Resource kind for HTTP Endpoints.
const HTTP_ENDPOINT_KIND: &str = "HttpEndpoint";

/// Derive a per-request [`Context`] and attach it to requests before they are handled.
pub struct ContextService<S> {
    authenticator: Authenticator,
    authoriser: Authoriser,
    config: ContextConfig,
    root: Context,
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for ContextService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        // Extract root context and optional middleware configuration.
        let root = request.app_data::<Data<Context>>();
        let config = request
            .app_data::<Data<ContextConfig>>()
            .map(|data| data.as_ref())
            .unwrap_or(&self.config)
            .clone();

        // Derive the per-request context.
        let context = root
            .map(|root| root.derive())
            .unwrap_or_else(|| self.root.derive());
        let pcontext = root
            .map(|root| root.as_ref().clone())
            .unwrap_or_else(|| self.root.clone());

        // Delay invoking the service so we can configure the request asynchronously.
        let authenticator = self.authenticator.clone();
        let authoriser = self.authoriser.clone();
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            let context = context_derive_logging(context, &config);
            let context = context_derive_auth(authenticator, &pcontext, context, &request).await;
            let context = context.map_err(replisdk::utils::actix::error::Error::from)?;
            let context = context.build();

            // Authorise the request with the newly derived context before processing it.
            authoriser
                .authorise(&context)
                .await
                .map_err(replisdk::utils::actix::error::Error::from)?;

            // Attach the derived context to the request.
            request.extensions_mut().insert(context);

            // Proceed to the wrapped service and handle the request.
            let service = service.call(request);
            service.await
        })
    }
}

/// Wrap an [`App`](actix_web::App) with a middleware that derives per-request contexts.
#[derive(Clone)]
pub struct ContextMiddleware {
    authenticator: Authenticator,
    authoriser: Authoriser,
    config: ContextConfig,
    root: Context,
}

impl ContextMiddleware {
    /// Initialise a [`ContextMiddleware`] with a root [`Context`] to use as a fallback.
    pub fn new(root: Context, authenticator: Authenticator, authoriser: Authoriser) -> Self {
        let config = ContextConfig::default();
        Self {
            authenticator,
            authoriser,
            config,
            root,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ContextMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ContextService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let middleware = ContextService {
            authenticator: self.authenticator.clone(),
            authoriser: self.authoriser.clone(),
            config: self.config.clone(),
            root: self.root.clone(),
            service: Arc::new(service),
        };
        std::future::ready(Ok(middleware))
    }
}

/// Configuration of the per-request [`Context`] derivation process.
#[derive(Clone, Debug)]
pub struct ContextConfig {
    /// Enable adding the current trace ID to logs (if a trace ID is available).
    pub add_trace_id: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        ContextConfig { add_trace_id: true }
    }
}

/// Configure authentication parameters for the derived context.
async fn context_derive_auth(
    authenticator: Authenticator,
    pcontext: &Context,
    context: ContextBuilder,
    request: &ServiceRequest,
) -> Result<ContextBuilder> {
    // Determine authorisation action from the request method.
    let action = request.method().to_string().to_ascii_lowercase();
    let action = Action::define("http", &action);

    // Determine the authorisation resource from the request URL.
    let mut metadata = BTreeMap::new();
    metadata.insert("uri".into(), request.uri().to_string());
    let resource_id = request
        .match_name()
        .map(Into::into)
        .or_else(|| request.match_pattern())
        .unwrap_or_else(|| request.uri().to_string());
    let resource = Resource {
        kind: HTTP_ENDPOINT_KIND.into(),
        metadata,
        resource_id,
    };

    // Determine the entity for the request using the configured authoriser.
    let entity = authenticator
        .authenticate(pcontext, request.request())
        .await?;

    // Combine the new information into the request context.
    let auth = AuthContext {
        action,
        entity,
        impersonate: None,
        resource,
    };
    Ok(context.authenticated(auth))
}

/// Configure logging options for the derived context.
fn context_derive_logging(context: ContextBuilder, config: &ContextConfig) -> ContextBuilder {
    if config.add_trace_id {
        context.log_trace()
    } else {
        context
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_and_read_body_json;
    use actix_web::test::init_service;
    use actix_web::test::TestRequest;
    use actix_web::FromRequest;
    use actix_web::HttpMessage;
    use actix_web::HttpResponse;

    use replicore_context::Context;

    fn factory(root: Context) -> super::ContextMiddleware {
        let injector = replicore_injector::Injector::fixture();
        super::ContextMiddleware::new(
            root,
            replicore_auth_insecure::Anonymous.into(),
            replicore_auth::access::Authoriser::wrap(
                replicore_auth_insecure::Unrestricted,
                injector.events.backend().into(),
            ),
        )
    }

    #[actix_web::get("/")]
    async fn inspect(_context: Context) -> HttpResponse {
        HttpResponse::Ok().json(43u64)
    }

    #[actix_web::test]
    async fn extract_context() {
        let context = Context::fixture();
        let request = TestRequest::get().to_http_request();
        request.extensions_mut().insert(context);
        Context::extract(&request).await.unwrap();
    }

    #[actix_web::test]
    async fn inject_context() {
        let root = Context::fixture();
        let app = actix_web::App::new().service(inspect).wrap(factory(root));
        let app = init_service(app).await;

        let request = TestRequest::get().uri("/").to_request();
        let response: u64 = call_and_read_body_json(&app, request).await;
        assert_eq!(response, 43u64);
    }
}
