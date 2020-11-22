use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_json::json;
use slog::debug;
use slog::Logger;
use uuid::Uuid;

use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Disapprove {
    data: DisapproveData,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Disapprove {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Disapprove {
        let data = DisapproveData {
            logger: logger.clone(),
            store: interfaces.stores.primary.clone(),
        };
        Disapprove {
            data,
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(
            logger,
            tracer,
            "/cluster/{cluster_id}/action/{action_id}/disapprove",
        );
        web::resource("/action/{action_id}/disapprove")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::post().to(responder))
    }
}

async fn responder(
    data: web::Data<DisapproveData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();
    let action_id = path
        .get("action_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("action_id"))?;
    let action_id = Uuid::parse_str(action_id)
        .with_context(|_| ErrorKind::APIRequestParameterInvalid("action_id"))?;

    let mut request = request;
    with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .actions(cluster_id.clone())
            .disapprove(action_id, span)
            .with_context(|_| ErrorKind::PrimaryStorePersist("action disapproval"))
    })?;

    debug!(
        data.logger,
        "Disapproved (rejected) action for scheduling";
        "cluster" => cluster_id,
        "action" => %action_id,
    );
    let response = HttpResponse::Ok().json(json!({}));
    Ok(response)
}

#[derive(Clone)]
struct DisapproveData {
    logger: Logger,
    store: Store,
}
