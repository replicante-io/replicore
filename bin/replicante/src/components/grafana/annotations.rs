use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;

use replicante_models_core::events::action::ActionEvent;
use replicante_models_core::events::agent::AgentEvent;
use replicante_models_core::events::cluster::ClusterEvent;
use replicante_models_core::events::discovery_settings::DiscoverySettingsEvent;
use replicante_models_core::events::node::NodeEvent;
use replicante_models_core::events::shard::ShardEvent;
use replicante_models_core::events::Event;
use replicante_models_core::events::Payload;
use replicante_store_view::store::events::EventsFilters;
use replicante_store_view::store::events::EventsOptions;
use replicante_store_view::store::Store;
use replicante_util_actixweb::with_request_span;

use crate::ErrorKind;
use crate::Interfaces;
use crate::Result;

pub struct Annotations {
    data: AnnotationData,
}

impl Annotations {
    pub fn new(interfaces: &mut Interfaces) -> Annotations {
        let data = AnnotationData {
            store: interfaces.stores.view.clone(),
        };
        Annotations { data }
    }

    pub fn resource(&self) -> impl HttpServiceFactory {
        web::resource("/annotations")
            .data(self.data.clone())
            .route(web::post().to(responder))
    }

    fn tags(event: &Event) -> Vec<String> {
        vec![
            event.code().into(),
            String::from(event.cluster_id().unwrap_or("System")),
        ]
    }

    fn text(event: &Event) -> String {
        match &event.payload {
            Payload::Action(action) => match action {
                ActionEvent::Changed(change) => format!(
                    "Details about action with ID {} on {} changed",
                    &change.current.action_id, &change.cluster_id,
                ),
                ActionEvent::Finished(action) => format!(
                    "Action with ID {} on {} was completed",
                    &action.action_id, &action.cluster_id,
                ),
                ActionEvent::Lost(action) => format!(
                    "Unfinished action with ID {} on {} was no longer reported by the agent",
                    &action.action_id, &action.cluster_id,
                ),
                ActionEvent::New(action) => format!(
                    "A new action with ID {} was created on {}",
                    &action.action_id, &action.cluster_id,
                ),
                _ => event.code().to_string(),
            },
            Payload::Agent(agent) => match agent {
                AgentEvent::Down(state) => {
                    format!("Agent {} is down or non-responsive", &state.host)
                }
                AgentEvent::InfoChanged(change) => {
                    format!("Details about agent on {} changed", &change.before.host)
                }
                AgentEvent::InfoNew(info) => {
                    format!("A new agent was detected on host {}", &info.host)
                }
                AgentEvent::New(agent) => {
                    format!("A new agent was detected on host {}", &agent.host)
                }
                AgentEvent::Up(change) => format!("Agent {} is now up", &change.host),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Cluster(cluster) => match cluster {
                ClusterEvent::Changed(_) => String::from(concat!(
                    "Cluster discovery record changed (most commonly, this",
                    " indicates a membership change)",
                )),
                ClusterEvent::New(_) => "Cluster discovered for the first time".into(),
                ClusterEvent::OrchestrateReport(report) => format!(
                    "A cluster orchestration report was emitted for cluster {}.{}",
                    report.namespace, report.cluster_id,
                ),
                ClusterEvent::SettingsSynthetic(settings) => format!(
                    "A synthetic ClusterSettings record was created for cluster {}.{}",
                    settings.namespace, settings.cluster_id,
                ),
                _ => event.code().to_string(),
            },
            Payload::DiscoverySettings(settings) => match settings {
                DiscoverySettingsEvent::Apply(settings) => format!(
                    "A DiscoverySettings object named {} was applied in {}",
                    &settings.name, &settings.namespace,
                ),
                DiscoverySettingsEvent::Delete(id) => format!(
                    "A DiscoverySettings object named {} was delete from {}",
                    &id.name, &id.namespace,
                ),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Node(node) => match node {
                NodeEvent::Changed(change) => {
                    format!("Details about datastore node {} changed", &change.node_id)
                }
                NodeEvent::Down(state) => format!(
                    "Node {} is down or non-responsive but the agent on the node could be reached",
                    &state.host,
                ),
                NodeEvent::New(_) => "A new datastore node was detected".into(),
                NodeEvent::Up(change) => format!("Datastore node {} is now up", &change.host),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Shard(shard) => match shard {
                ShardEvent::AllocationChanged(change) => format!(
                    "Status of shard {} on node {} have changed",
                    &change.shard_id, &change.node_id
                ),
                ShardEvent::AllocationNew(shard) => format!(
                    "Shard {} found on node {} for the first time",
                    &shard.shard_id, &shard.node_id
                ),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            // TODO: for when #[non_exhaustive] is usable
            //_ => event.code().to_string(),
        }
    }

    fn title(event: &Event) -> String {
        match &event.payload {
            Payload::Action(action) => match action {
                ActionEvent::Changed(_) => "Action details changed".into(),
                ActionEvent::Finished(_) => "Action finished executing".into(),
                ActionEvent::Lost(_) => "Unfinished action is no longer reported".into(),
                ActionEvent::New(_) => "New action detected".into(),
                _ => event.code().to_string(),
            },
            Payload::Agent(agent) => match agent {
                AgentEvent::Down(_) => "Agent is down".into(),
                AgentEvent::InfoChanged(_) => "Agent details changed".into(),
                AgentEvent::InfoNew(_) => "New agent detected".into(),
                AgentEvent::New(_) => "New agent detected".into(),
                AgentEvent::Up(_) => "Agent is up".into(),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Cluster(cluster) => match cluster {
                ClusterEvent::Changed(_) => "Cluster changed".into(),
                ClusterEvent::New(_) => "New cluster detected".into(),
                ClusterEvent::OrchestrateReport(_) => {
                    "Cluster orchestration task emitted a report".into()
                }
                ClusterEvent::SettingsSynthetic(_) => "Synthetic ClusterSettings created".into(),
                _ => event.code().to_string(),
            },
            Payload::DiscoverySettings(settings) => match settings {
                DiscoverySettingsEvent::Apply(_) => "DiscoverySettings applied".into(),
                DiscoverySettingsEvent::Delete(_) => "DiscoverySettings deleted".into(),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Node(node) => match node {
                NodeEvent::Changed(_) => "Datastore node details changed".into(),
                NodeEvent::Down(_) => "Datastore node is down".into(),
                NodeEvent::New(_) => "New datastore node detected".into(),
                NodeEvent::Up(_) => "Datastore node is up".into(),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            Payload::Shard(shard) => match shard {
                ShardEvent::AllocationChanged(_) => "Shard status on node changed".into(),
                ShardEvent::AllocationNew(_) => "Shard found on node".into(),
                // TODO: for when #[non_exhaustive] is usable
                //_ => event.code().to_string(),
            },
            // TODO: for when #[non_exhaustive] is usable
            //_ => event.code().to_string(),
        }
    }
}

#[derive(Clone)]
struct AnnotationData {
    store: Store,
}

/// Advanced query parameters passed as JSON blob in the annotation.query field.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct AdvancedQuery {
    #[serde(default)]
    cluster_id: Option<String>,

    #[serde(default)]
    event: Option<String>,

    #[serde(default = "AdvancedQuery::default_exclude_system_events")]
    exclude_system_events: bool,

    #[serde(default = "AdvancedQuery::default_limit")]
    limit: i64,
}

impl Default for AdvancedQuery {
    fn default() -> Self {
        Self {
            cluster_id: None,
            event: None,
            exclude_system_events: Self::default_exclude_system_events(),
            limit: Self::default_limit(),
        }
    }
}

impl AdvancedQuery {
    fn default_exclude_system_events() -> bool {
        false
    }
    fn default_limit() -> i64 {
        1000
    }
}

/// Response annotation, a list of which is our response to SimpleJson.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct Annotation {
    tags: Vec<String>,
    text: String,
    time: i64,
    title: String,
}

/// Request data sent to us by SimpleJson.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct AnnotationRequest {
    annotation: AnnotationQuery,
    range: AnnotationRequestRange,
}

/// Annotation query sent by SimpleJson.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct AnnotationQuery {
    datasource: String,
    enable: bool,
    #[serde(rename = "iconColor")]
    icon_color: String,
    name: String,
    query: Option<String>,
}

/// Time-range for the annotation query.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct AnnotationRequestRange {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

async fn responder(
    body: web::Json<AnnotationRequest>,
    data: web::Data<AnnotationData>,
    request: HttpRequest,
) -> Result<impl Responder> {
    // We should not get queries for disabled annotations but just in case skip them.
    if !body.annotation.enable {
        let response = HttpResponse::Ok().json(json!([]));
        return Ok(response);
    }

    // Build request filters.
    let query = match body.annotation.query.as_ref() {
        None => AdvancedQuery::default(),
        Some(query) if query.is_empty() => AdvancedQuery::default(),
        Some(query) => {
            serde_json::from_str(query).with_context(|_| ErrorKind::APIRequestBodyInvalid)?
        }
    };
    let filters = EventsFilters {
        cluster_id: query.cluster_id,
        event: query.event,
        exclude_system_events: query.exclude_system_events,
        start_from: Some(body.range.from),
        stop_at: Some(body.range.to),
    };
    let options = EventsOptions {
        limit: Some(query.limit),
        ..EventsOptions::default()
    };

    // Fetch and format annotations.
    let mut request = request;
    let events = with_request_span(&mut request, |span| {
        let span = span.map(|span| span.context().clone());
        data.store
            .events()
            .range(filters, options, span)
            .with_context(|_| ErrorKind::ViewStoreQuery("events"))
    })?;
    let mut annotations: Vec<Annotation> = Vec::new();
    for event in events {
        let event = event.with_context(|_| ErrorKind::Deserialize("event record", "Event"))?;
        let tags = Annotations::tags(&event);
        let text = Annotations::text(&event);
        let time = event.timestamp.timestamp_millis();
        let title = Annotations::title(&event);
        annotations.push(Annotation {
            tags,
            text,
            time,
            title,
        });
    }

    // Send the response to clients.
    let response = HttpResponse::Ok().json(annotations);
    Ok(response)
}
