use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Plugin;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use uuid::Uuid;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_store_view::store::actions::SearchFilters;
use replicante_store_view::store::Store;
use replicante_util_iron::request_span;

use crate::Error;
use crate::ErrorKind;

/// Cluster action details (`/webui/cluster/:cluster/action/:action`) handler.
pub struct ActionDetails {
    store: Store,
}

impl ActionDetails {
    pub fn new(store: Store) -> Self {
        ActionDetails { store }
    }
}

impl Handler for ActionDetails {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Get cluster and action IDs.
        let cluster_id = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let action_id = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("action")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("action"))
            .map_err(Error::from)?;
        let action_id = Uuid::from_str(&action_id)
            .with_context(|_| ErrorKind::APIRequestParameterInvalid("action"))
            .map_err(Error::from)?;
        let span = request_span(req);

        // Fetch requested action information.
        let store = self.store.actions(cluster_id);
        let action = store
            .action(action_id, span.context().clone())
            .with_context(|_| ErrorKind::ViewStoreQuery("action"))
            .map_err(Error::from)?;
        let action = match action {
            Some(action) => action,
            None => {
                let mut resp = Response::new();
                resp.set_mut(JsonResponse::json(json!({})))
                    .set_mut(status::NotFound);
                return Ok(resp);
            }
        };
        let history = store
            .history(action_id, span.context().clone())
            .with_context(|_| ErrorKind::ViewStoreQuery("action history"))
            .map_err(Error::from)?;

        // Send the response.
        let details = ActionDetailsResponse { action, history };
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(details))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

/// Actions details returned by the API.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct ActionDetailsResponse {
    pub action: Action,
    pub history: Vec<ActionHistory>,
}

/// Cluster actions (`/webui/cluster/:cluster/actions`) handler.
pub struct Actions {
    store: Store,
}

impl Actions {
    pub fn new(store: Store) -> Self {
        Actions { store }
    }
}

impl Handler for Actions {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Get cluster ID and search filters.
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let filters = req
            .get::<bodyparser::Struct<ActionsSearch>>()
            .with_context(|_| ErrorKind::APIRequestBodyInvalid)
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::APIRequestBodyNotFound)
            .map_err(Error::from)?;

        // Search for actions and prepare results.
        let filters = SearchFilters::from(filters);
        let span = request_span(req);
        let cursor = self
            .store
            .actions(cluster)
            .search(filters, span.context().clone())
            .with_context(|_| ErrorKind::ViewStoreQuery("actions"))
            .map_err(Error::from)?;

        let mut actions: Vec<Action> = Vec::new();
        for action in cursor {
            let action = action
                .with_context(|_| ErrorKind::Deserialize("action record", "Action"))
                .map_err(Error::from)?;
            actions.push(action);
        }

        // Send the response.
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(actions))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

/// Filters to search actions to be returned.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct ActionsSearch {
    action_kind: String,
    action_state: String,
    from: DateTime<Utc>,
    node_id: String,
    until: DateTime<Utc>,
}

impl From<ActionsSearch> for SearchFilters {
    fn from(filters: ActionsSearch) -> SearchFilters {
        let action_kind = if filters.action_kind.is_empty() {
            None
        } else {
            Some(filters.action_kind)
        };
        let action_state = if filters.action_state.is_empty() {
            None
        } else {
            Some(filters.action_state)
        };
        let node_id = if filters.node_id.is_empty() {
            None
        } else {
            Some(filters.node_id)
        };
        SearchFilters {
            action_kind,
            action_state,
            from: filters.from,
            node_id,
            until: filters.until,
        }
    }
}
