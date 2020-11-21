use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_derive::Serialize;
use slog::Logger;

use replicante_service_tasks::TaskRequest;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

use replicore_models_tasks::payload::ClusterRefreshPayload;
use replicore_models_tasks::ReplicanteQueues;
use replicore_models_tasks::Tasks;

use crate::interfaces::Interfaces;
use crate::ErrorKind;
use crate::Result;

pub struct Refresh {
    data: RefreshData,
}

impl Refresh {
    pub fn new(logger: &Logger, interfaces: &mut Interfaces) -> Refresh {
        let data = RefreshData {
            logger: logger.clone(),
            store: interfaces.stores.primary.clone(),
            tasks: interfaces.tasks.clone(),
            tracer: interfaces.tracing.tracer(),
        };
        Refresh { data }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.data.logger.clone();
        let tracer = Arc::clone(&self.data.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/refresh");
        web::resource("/refresh")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::post().to(responder))
    }
}

async fn responder(data: web::Data<RefreshData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    let mut request = request;
    let cluster = with_request_span(&mut request, |span| -> Result<_> {
        let span = span.map(|span| span.context().clone());
        let cluster = data
            .store
            .cluster("TODO_NS".to_string(), cluster_id.clone())
            .discovery(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_discovery"))?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_discovery", cluster_id.clone()))?;
        Ok(cluster)
    })?;

    let payload = ClusterRefreshPayload::new(cluster, false);
    let mut task = TaskRequest::new(ReplicanteQueues::ClusterRefresh);
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

    let response = RefreshResponse { task_id };
    let response = HttpResponse::Ok().json(response);
    Ok(response)
}

#[derive(Clone)]
struct RefreshData {
    logger: Logger,
    store: Store,
    tasks: Tasks,
    tracer: Arc<opentracingrust::Tracer>,
}

/// Cluster refresh response.
#[derive(Serialize)]
struct RefreshResponse {
    task_id: String,
}
