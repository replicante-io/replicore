use std::collections::HashMap;

use failure::ResultExt;
use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use router::Router;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::Agent;
use replicante_models_core::AgentInfo;
use replicante_models_core::AgentStatus;
use replicante_store_primary::store::Store as PrimaryStore;
use replicante_store_view::store::events::EventsFilters;
use replicante_store_view::store::events::EventsOptions;
use replicante_store_view::store::Store as ViewStore;
use replicante_util_iron::request_span;

use super::constants::RECENT_EVENTS_LIMIT;
use crate::Error;
use crate::ErrorKind;

/// Agent details returned by the API.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct AgentDetails {
    pub host: String,
    pub status: AgentStatus,
    pub version_checkout: Option<String>,
    pub version_number: Option<String>,
    pub version_taint: Option<String>,
}

impl AgentDetails {
    pub fn combine(agent: Agent, details: Option<AgentInfo>) -> AgentDetails {
        let mut record = AgentDetails {
            host: agent.host,
            status: agent.status,
            version_checkout: None,
            version_number: None,
            version_taint: None,
        };
        if let Some(details) = details {
            record.version_checkout = Some(details.version_checkout);
            record.version_number = Some(details.version_number);
            record.version_taint = Some(details.version_taint);
        }
        record
    }
}

/// Cluster agents (`/webui/cluster/:cluster/agents`) handler.
pub struct Agents {
    store: PrimaryStore,
}

impl Handler for Agents {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let span = request_span(req);

        // Start fetching all agents and their status.
        let mut agents = Vec::new();
        let iter = self
            .store
            .agents(cluster.clone())
            .iter(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("agents.iter"))
            .map_err(Error::from)?;
        for agent in iter {
            let agent = agent
                .with_context(|_| ErrorKind::Deserialize("agent record", "Agent"))
                .map_err(Error::from)?;
            agents.push(agent);
        }

        // Fetch all agents information, indexed by host.
        let mut details = HashMap::new();
        let iter = self
            .store
            .agents(cluster.clone())
            .iter_info(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("agents.iter_info"))
            .map_err(Error::from)?;
        for agent in iter {
            let agent = agent
                .with_context(|_| ErrorKind::Deserialize("agent information record", "AgentInfo"))
                .map_err(Error::from)?;
            details.insert(agent.host.clone(), agent);
        }

        // Combine everything into the API format and return a list of agents.
        let agents: Vec<AgentDetails> = agents
            .into_iter()
            .map(|agent| {
                let info = details.remove(&agent.host);
                AgentDetails::combine(agent, info)
            })
            .collect();
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(agents)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Agents {
    pub fn new(store: PrimaryStore) -> Self {
        Agents { store }
    }
}

/// Cluster discovery (`/webui/cluster/:cluster/discovery`) handler.
pub struct Discovery {
    store: PrimaryStore,
}

impl Handler for Discovery {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let span = request_span(req);
        let discovery = self
            .store
            .cluster(cluster.clone())
            .discovery(span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster.discovery"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("ClusterDiscovery", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(discovery))
            .set_mut(status::Ok);
        Ok(resp)
    }
}

impl Discovery {
    pub fn new(store: PrimaryStore) -> Self {
        Discovery { store }
    }
}

/// Cluster events (`/webui/cluster/:cluster/events`) handler.
pub struct Events {
    store: ViewStore,
}

impl Handler for Events {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;

        let mut filters = EventsFilters::all();
        filters.cluster_id = Some(cluster);
        let mut options = EventsOptions::default();
        options.limit = Some(RECENT_EVENTS_LIMIT);
        options.reverse = true;

        let span = request_span(req);
        let iter = self
            .store
            .events()
            .range(filters, options, span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events.range"))
            .map_err(Error::from)?;
        let mut events = Vec::new();
        for event in iter {
            let event = event
                .with_context(|_| ErrorKind::Deserialize("event record", "Event"))
                .map_err(Error::from)?;
            events.push(event);
        }

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(events)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Events {
    pub fn new(store: ViewStore) -> Self {
        Events { store }
    }
}

/// Cluster meta (`/webui/cluster/:cluster/meta`) handler.
pub struct Meta {
    store: PrimaryStore,
}

impl Handler for Meta {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let cluster = req
            .extensions
            .get::<Router>()
            .expect("Iron Router extension not found")
            .find("cluster")
            .map(String::from)
            .ok_or_else(|| ErrorKind::APIRequestParameterNotFound("cluster"))
            .map_err(Error::from)?;
        let span = request_span(req);
        let meta = self
            .store
            .legacy()
            .cluster_meta(cluster.clone(), span.context().clone())
            .with_context(|_| ErrorKind::PrimaryStoreQuery("cluster_meta"))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::ModelNotFound("cluster_meta", cluster))
            .map_err(Error::from)?;
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(meta)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Meta {
    pub fn new(store: PrimaryStore) -> Self {
        Meta { store }
    }
}
