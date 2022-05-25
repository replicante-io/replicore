use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_json::json;
use slog::Logger;

use replicante_models_core::events::Event;
use replicante_models_core::scope::Namespace;
use replicante_store_primary::store::Store;
use replicante_stream_events::EmitMessage;
use replicante_stream_events::Stream;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Delete {
    data: DeleteData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Delete {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Delete {
        let data = DeleteData {
            events: interfaces.streams.events.clone(),
            store: interfaces.stores.primary.clone(),
        };
        Delete {
            data,
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(
            logger,
            tracer,
            "/discoverysettings/{namespace}/{name}/delete",
        );
        web::resource("/{name}/delete")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::delete().to(responder))
    }
}

async fn responder(data: web::Data<DeleteData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let namespace = path
        .get("namespace")
        .ok_or(ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();
    let name = path
        .get("name")
        .ok_or(ErrorKind::APIRequestParameterNotFound("name"))?
        .to_string();

    // TODO(namespace-rollout): Replace this check with NS lookup.
    if namespace != Namespace::HARDCODED_FOR_ROLLOUT().ns_id {
        let error = ErrorKind::NamespaceRolloutNotDefault(namespace);
        return Err(error.into());
    }

    let mut request = request;
    with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        let event = Event::builder()
            .discovery_settings()
            .delete(namespace.clone(), name.clone());
        let code = event.code();
        let stream_key = event.entity_id().partition_key();
        let event = EmitMessage::with(stream_key, event)
            .with_context(|_| ErrorKind::EventsStreamEmit(code))?
            .trace(span.clone());
        data.events
            .emit(event)
            .with_context(|_| ErrorKind::EventsStreamEmit("DiscoverySettings"))?;
        data.store
            .discovery_settings(namespace)
            .delete(&name, span)
            .with_context(|_| ErrorKind::PrimaryStoreDelete("discovery settings"))
    })?;

    let response = HttpResponse::Ok().json(json!({}));
    Ok(response)
}

#[derive(Clone)]
struct DeleteData {
    events: Stream,
    store: Store,
}
