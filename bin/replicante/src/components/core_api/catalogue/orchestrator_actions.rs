use std::collections::BTreeMap;
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use slog::Logger;

use replicante_util_actixweb::TracingMiddleware;
use replicore_iface_orchestrator_action::OrchestratorActionDescriptor;
use replicore_iface_orchestrator_action::OrchestratorActionRegistry;

use crate::interfaces::Interfaces;
use crate::Result;

pub struct OrchestratorActions {
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl OrchestratorActions {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> OrchestratorActions {
        OrchestratorActions {
            logger: logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/catalogue/orchestrator-actions");
        web::resource("/orchestrator-actions")
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

async fn responder() -> Result<impl Responder> {
    let registry = OrchestratorActionRegistry::current();
    let catalogue: BTreeMap<&str, OrchestratorActionDescriptor> = registry
        .iter()
        .map(|(id, handler)| (id, handler.describe()))
        .collect();
    let response = HttpResponse::Ok().json(catalogue);
    Ok(response)
}
