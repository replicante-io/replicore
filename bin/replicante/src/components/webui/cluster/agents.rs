use std::collections::HashMap;
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use slog::Logger;

use replicante_models_core::agent::Agent;
use replicante_models_core::agent::AgentInfo;
use replicante_models_core::agent::AgentStatus;
use replicante_store_primary::store::Store;
use replicante_util_actixweb::with_request_span;
use replicante_util_actixweb::TracingMiddleware;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct Agents {
    data: AgentsData,
    logger: Logger,
    tracer: Arc<opentracingrust::Tracer>,
}

impl Agents {
    pub fn new(interfaces: &mut Interfaces) -> Agents {
        let data = AgentsData {
            store: interfaces.stores.primary.clone(),
        };
        Agents {
            data,
            logger: interfaces.logger.clone(),
            tracer: interfaces.tracing.tracer(),
        }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        let logger = self.logger.clone();
        let tracer = Arc::clone(&self.tracer);
        let tracer = TracingMiddleware::with_name(logger, tracer, "/cluster/{cluster_id}/agents");
        web::resource("/agents")
            .data(self.data.clone())
            .wrap(tracer)
            .route(web::get().to(responder))
    }
}

#[derive(Clone)]
struct AgentsData {
    store: Store,
}

async fn responder(data: web::Data<AgentsData>, request: HttpRequest) -> Result<impl Responder> {
    let path = request.match_info();
    let cluster_id = path
        .get("cluster_id")
        .ok_or(ErrorKind::APIRequestParameterNotFound("cluster_id"))?
        .to_string();

    // Start fetching all agents and their status.
    let mut agents = Vec::new();
    let mut request = request;
    let iter = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .agents(cluster_id.clone())
            .iter(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("agents.iter"))
    })?;
    for agent in iter {
        let agent = agent.with_context(|_| ErrorKind::Deserialize("agent record", "Agent"))?;
        agents.push(agent);
    }

    // Fetch all agents information, indexed by host.
    let mut details = HashMap::new();
    let iter = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .agents(cluster_id)
            .iter_info(span)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("agents.iter_info"))
    })?;
    for agent in iter {
        let agent = agent
            .with_context(|_| ErrorKind::Deserialize("agent information record", "AgentInfo"))?;
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
    let response = HttpResponse::Ok().json(agents);
    Ok(response)
}

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
