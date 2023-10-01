//! ActixWeb Middleware to attach [`Context`] objects to requests.
use std::future::Ready;

use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpMessage;

use super::Context;

/// Derive a per-request [`Context`] and attach it to requests before they are handled.
pub struct ContextService<S> {
    root: Context,
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContextService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        // Extract root context and optional middleware configuration.
        let root = request.app_data::<Data<Context>>();
        let config = request.app_data::<Data<ContextConfig>>();

        // Derive the per-request context.
        let mut context = root
            .map(|root| root.derive())
            .unwrap_or_else(|| self.root.derive());

        let add_trace_id = config
            .as_ref()
            .map(|config| config.add_trace_id)
            .unwrap_or(true);
        if add_trace_id {
            context = context.log_trace();
        }

        // Attach the derived context to the request.
        let context = context.build();
        request.extensions_mut().insert(context);

        // Proceed to the wrapped service and handle the request.
        self.service.call(request)
    }
}

/// Wrap an [`App`](actix_web::App) with a middleware that derives per-request contexts.
pub struct ContextMiddleware {
    root: Context,
}

impl ContextMiddleware {
    /// Initialise a [`ContextMiddleware`] with a root [`Context`] to use as a fallback.
    pub fn new(context: Context) -> Self {
        Self { root: context }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ContextMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
            root: self.root.clone(),
            service,
        };
        std::future::ready(Ok(middleware))
    }
}

/// Configuration of the per-request [`Context`] derivation process.
pub struct ContextConfig {
    add_trace_id: bool,
}

impl ContextConfig {
    /// Enable or disable adding the current trace ID to logs (if a trace ID is available).
    pub fn add_trace_id(mut self, add: bool) -> Self {
        self.add_trace_id = add;
        self
    }

    /// Initialise a default configuration.
    pub fn new() -> Self {
        ContextConfig::default()
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        ContextConfig { add_trace_id: true }
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

    use super::super::Context;

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
        let app = actix_web::App::new()
            .service(inspect)
            .wrap(super::ContextMiddleware::new(root));
        let app = init_service(app).await;

        let request = TestRequest::get().uri("/").to_request();
        let response: u64 = call_and_read_body_json(&app, request).await;
        assert_eq!(response, 43u64);
    }
}
