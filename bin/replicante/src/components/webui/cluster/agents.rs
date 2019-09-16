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

use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_store_primary::store::Store as PrimaryStore;
use replicante_util_iron::request_span;

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
