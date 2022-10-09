use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use slog::Logger;

use replicante_models_core::api::discovery_settings::DiscoverySettingsListResponse;
use replicante_models_core::scope::Namespace;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct List {
    data: web::Data<ListData>,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl List {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> List {
        let data = ListData {
            store: interfaces.stores.primary.clone(),
        };
        List {
            data: web::Data::new(data),
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/discoverysettings/{namespace}/list");
        web::resource("/list")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder(data: web::Data<ListData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let namespace = path
        .get("namespace")
        .ok_or(ErrorKind::APIRequestParameterNotFound("namespace"))?
        .to_string();

    // TODO(namespace-rollout): Replace this check with NS lookup.
    if namespace != Namespace::HARDCODED_FOR_ROLLOUT().ns_id {
        let error = ErrorKind::NamespaceRolloutNotDefault(namespace);
        return Err(error.into());
    }

    let mut request = request;
    let cursor = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .discovery_settings(namespace)
            .iter_names(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("discovery settings"))
    })?;

    let mut names = vec![];
    for name in cursor {
        let name = name.with_context(|_| ErrorKind::PrimaryStoreQuery("discovery settings"))?;
        names.push(name);
    }

    let response = DiscoverySettingsListResponse { names };
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}

#[derive(Clone)]
struct ListData {
    store: Store,
}
