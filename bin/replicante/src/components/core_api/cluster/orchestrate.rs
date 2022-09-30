use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde::Serialize;
use slog::Logger;

use replicante_models_core::scope::Namespace;
use replicante_service_tasks::TaskRequest;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use replicore_models_tasks::payload::OrchestrateClusterPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Tasks;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Orchestrate {
    data: web::Data<OrchestrateData>,
}

impl Orchestrate {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Orchestrate {
        let data = OrchestrateData {
            logger: logger.clone(),
            store: interfaces.stores.primary.clone(),
            tasks: interfaces.tasks.clone(),
            tracer: interfaces.tracing.tracer(),
        };
        let data = web::Data::new(data);
        Orchestrate { data }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.data.tracer);
        let tracer =
            TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/orchestrate");
        web::resource("/orchestrate")
            .app_data(self.data.clone())
            .wrap(tracer)
            .route(web::post().to(responder))
    }
}

async fn responder(
    data: web::Data<OrchestrateData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut request = request;
    let namespace = Namespace::HARDCODED_FOR_ROLLOUT().ns_id;

    // Check the cluster exists before scheduling a task for it.
    with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let cluster = data
            .store
            .cluster(namespace.clone(), cluster_id.clone())
            .discovery(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_discovery"))?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_discovery", cluster_id.clone()))?;
        Ok(cluster)
    })?;

    let payload = OrchestrateClusterPayload::new(namespace, &cluster_id);
    let mut task = TaskRequest::new(ReplicanteQueues::OrchestrateCluster);
    with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context());
        if let Some(span) = span {
            if let Err(error) = task.trace(span, &data.tracer) {
                let error = failure::SyncFailure::new(error);
                capture_fail!(
                    &error,
                    data.logger,
                    "Unable to inject trace context in task request";
                    "cluster_id" => &cluster_id,
                    failure_info(&error),
                );
            }
        }
    });

    let task_id = task.id().to_string();
    if let Err(error) = data.tasks.request(task, payload) {
        capture_fail!(
            &error,
            data.logger,
            "Failed to request cluster refresh";
            "cluster_id" => &cluster_id,
            failure_info(&error),
        );
    };

    let response = OrchestrateResponse { task_id };
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}

#[derive(Clone)]
struct OrchestrateData {
    logger: Logger,
    store: Store,
    tasks: Tasks,
    tracer: Arc<opentracingrust::Tracer>,
}

/// Cluster orchestrate response.
#[derive(Serialize)]
struct OrchestrateResponse {
    task_id: String,
}
