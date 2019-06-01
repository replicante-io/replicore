use std::sync::Arc;

use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use opentracingrust::Tracer;
use router::Router;
use serde_derive::Serialize;
use slog::Logger;

use replicante_data_store::store::Store;
use replicante_tasks::TaskRequest;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_iron::request_span;

use super::super::super::interfaces::api::APIRoot;
use super::super::super::interfaces::Interfaces;
use super::super::super::task_payload::ClusterRefreshPayload;
use super::super::super::tasks::ReplicanteQueues;
use super::super::super::tasks::Tasks;
use super::super::super::Error;
use super::super::super::ErrorKind;

/// Attach cluster-related core API handlers.
pub fn attach(logger: Logger, interfaces: &mut Interfaces) {
    let store = interfaces.store.clone();
    let tasks = interfaces.tasks.clone();
    let tracer = interfaces.tracing.tracer();
    let mut router = interfaces.api.router_for(&APIRoot::UnstableCoreApi);
    let handler = Refresh {
        logger,
        store,
        tasks,
        tracer,
    };
    router.post(
        "/cluster/:cluster_id/refresh",
        handler,
        "/cluster/:cluster_id/refresh",
    );
}

/// Schedule a ClusterRefresh task.
pub struct Refresh {
    logger: Logger,
    store: Store,
    tasks: Tasks,
    tracer: Arc<Tracer>,
}

impl Handler for Refresh {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let cluster_id = request
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster_id")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster_id"))
            .map_err(Error::from)?;
        let span = request_span(request);
        let cluster = self
            .store
            .cluster(cluster_id.clone())
            .discovery(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_discovery"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_discovery", cluster_id.clone()))
            .map_err(Error::from)?;

        let payload = ClusterRefreshPayload::new(cluster, false);
        let mut task = TaskRequest::new(ReplicanteQueues::ClusterRefresh);
        if let Err(error) = task.trace(span.context(), &self.tracer) {
            let error = failure::SyncFailure::new(error);
            capture_fail!(
                &error,
                self.logger,
                "Unable to inject trace context in task request";
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        }
        let task_id = task.id().to_string();
        if let Err(error) = self.tasks.request(task, payload) {
            capture_fail!(
                &error,
                self.logger,
                "Failed to request cluster refresh";
                "cluster_id" => &cluster_id,
                failure_info(&error),
            );
        };

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(RefreshResponse { task_id }))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

/// Cluster refresh response.
#[derive(Serialize)]
struct RefreshResponse {
    task_id: String,
}
