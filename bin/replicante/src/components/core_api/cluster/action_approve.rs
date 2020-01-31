use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;
use serde_json::json;
use slog::debug;
use slog::Logger;
use uuid::Uuid;

use replicante_store_primary::store::Store;
use replicante_util_iron::request_span;

use crate::interfaces::api::APIRoot;
use crate::interfaces::Interfaces;
use crate::Error;
use crate::ErrorKind;

pub fn attach(logger: Logger, interfaces: &mut Interfaces) {
    let store = interfaces.stores.primary.clone();
    let mut router = interfaces.api.router_for(&APIRoot::UnstableCoreApi);
    let handler = Approve { logger, store };
    router.post(
        "/cluster/:cluster_id/action/:action_id/approve",
        handler,
        "/cluster/:cluster_id/action/:action_id/approve",
    );
}

struct Approve {
    logger: Logger,
    store: Store,
}

impl Handler for Approve {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let cluster_id = request
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster_id")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster_id"))
            .map_err(Error::from)?;
        let action_id = request
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("action_id")
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("action_id"))
            .map_err(Error::from)
            .and_then(|action| {
                Uuid::parse_str(action)
                    .with_context(|_| ErrorKind::APIRequestParameterInvalid("action_id"))
                    .map_err(Error::from)
            })?;

        let span = request_span(request);
        self.store
            .actions(cluster_id.clone())
            .approve(action_id, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStorePersist("action approval"))
            .map_err(Error::from)?;
        debug!(
            self.logger,
            "Approved action for scheduling";
            "cluster" => cluster_id,
            "action" => %action_id,
        );

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(json!({})))
            .set_mut(status::Ok);
        Ok(resp)
    }
}
