//! Module to define cluster related WebUI endpoints.
use bodyparser;
use chrono::DateTime;
use chrono::Utc;

use failure::ResultExt;
use failure::err_msg;

use iron::Handler;
use iron::IronResult;
use iron::Plugin;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use iron_json_response::JsonResponse;

use serde_json;

use replicante_data_models::Event;
use replicante_data_models::EventPayload;
use replicante_streams_events::EventsStream;
use replicante_streams_events::ScanFilters;
use replicante_streams_events::ScanOptions;

use super::super::super::Error;
use super::super::super::ErrorKind;
use super::Interfaces;


/// Advanced query parameters passed as JSON blob in the annotation.query field.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct AdvancedQuery {
    #[serde(default)]
    cluster_id: Option<String>,

    #[serde(default)]
    event: Option<String>,

    #[serde(default = "AdvancedQuery::default_exclude_snapshots")]
    exclude_snapshots: bool,

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
            exclude_snapshots: Self::default_exclude_snapshots(),
            exclude_system_events: Self::default_exclude_system_events(),
            limit: Self::default_limit(),
        }
    }
}

impl AdvancedQuery {
    fn default_exclude_snapshots() -> bool { true }
    fn default_exclude_system_events() -> bool { false }
    fn default_limit() -> i64 { 1000 }
}


/// Response annotation, a list of which is our response to SimpleJson.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct Annotation {
    tags: Vec<String>,
    text: String,
    time: i64,
    title: String,
}


/// Request data sent to us by SimpleJson.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct AnnotationRequest {
    annotation: AnnotationQuery,
    range: AnnotationRequestRange,
}


/// Annotation query sent by SimpleJson.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct AnnotationQuery {
    datasource: String,
    enable: bool,
    #[serde(rename = "iconColor")]
    icon_color: String,
    name: String,
    query: Option<String>,
}


/// Time-range for the annotation query.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
struct AnnotationRequestRange {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}


/// Grafana check endpoint (`/api/v1/grafana/annotations`) handler.
pub struct Annotations {
    events: EventsStream,
}

impl Handler for Annotations {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Get the annotation query.
        let request = req.get::<bodyparser::Struct<AnnotationRequest>>()
            .context(ErrorKind::Legacy(err_msg("failed to parse annotation request")))
            .map_err(Error::from)?
            .ok_or_else(|| ErrorKind::Legacy(err_msg("annotation request body missing")))
            .map_err(Error::from)?;

        // We should not get queries for disabled annotations but just in case skip them.
        if !request.annotation.enable {
            let mut resp = Response::new();
            let nothing: Vec<Annotation> = Vec::new();
            resp.set_mut(JsonResponse::json(nothing)).set_mut(status::Ok);
            return Ok(resp);
        }

        // Fetch and collect annotations.
        let query = match request.annotation.query.as_ref() {
            Some(query) if query == "" => AdvancedQuery::default(),
            Some(query) => serde_json::from_str(query)
                .context(ErrorKind::Legacy(err_msg("failed to parse annotation query")))
                .map_err(Error::from)?,
            None => AdvancedQuery::default(),
        };
        let mut filters = ScanFilters::most();
        let mut options = ScanOptions::default();
        filters.cluster_id = query.cluster_id;
        filters.event = query.event;
        filters.exclude_snapshots = query.exclude_snapshots;
        filters.exclude_system_events = query.exclude_system_events;
        filters.start_from = Some(request.range.from);
        filters.stop_at = Some(request.range.to);
        options.limit = Some(query.limit);
        let events = self.events.scan(filters, options).map_err(Error::from)
            .context(ErrorKind::Legacy(err_msg("failed to scan event stream")))
            .map_err(Error::from)?;
        let mut annotations: Vec<Annotation> = Vec::new();
        for event in events {
            let event = event.map_err(Error::from)
                .context(ErrorKind::Legacy(err_msg("failed to decode event from stream")))
                .map_err(Error::from)?;
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
        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(annotations)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Annotations {
    pub fn attach(interfaces: &mut Interfaces) {
        let router = interfaces.api.router();
        let handler = Annotations {
            events: interfaces.streams.events.clone(),
        };
        router.post("/api/v1/grafana/annotations", handler, "api/v1/grafana/annotations");
    }

    fn tags(event: &Event) -> Vec<String> {
        let mut tags = Vec::new();
        tags.push(event.code().into());
        tags.push(String::from(event.cluster().unwrap_or("System")));
        tags
    }

    fn text(event: &Event) -> String {
        match event.payload {
            EventPayload::AgentDown(ref data) => format!(
                "Agent {} is down or non-responsive", data.host
            ),
            EventPayload::AgentInfoChanged(ref data) => format!(
                "Details about agent on {} changed", data.before.host
            ),
            EventPayload::AgentInfoNew(ref data) => format!(
                "A new agent was detected on host {}", data.host
            ),
            EventPayload::AgentNew(ref data) => format!(
                "A new agent was detected on host {}", data.host
            ),
            EventPayload::AgentUp(ref data) => format!("Agent {} is now up", data.host),
            EventPayload::ClusterChanged(_) => String::from(concat!(
                "Cluster discovery record changed (most commonly, this indicates",
                "a membership change)"
            )),
            EventPayload::ClusterNew(_) => String::from("Cluster discovered for the first time"),
            EventPayload::NodeChanged(ref data) => format!(
                "Details about datastore node {} changed", data.host
            ),
            EventPayload::NodeDown(ref data) => format!(
                "Node {} is down or non-responsive but the agent on the node could be reached",
                data.host
            ),
            EventPayload::NodeNew(_) => "A new datastore node was detected".into(),
            EventPayload::NodeUp(ref data) => format!("Datastore node {} is now up", data.host),
            EventPayload::ShardAllocationChanged(ref data) => format!(
                "Status of shard {} on node {} have changed", data.id, data.node
            ),
            EventPayload::ShardAllocationNew(ref data) => format!(
                "Shard {} found on node {} for the first time", data.id, data.node
            ),
            _ => format!("{}", event.code()),
        }
    }

    fn title(event: &Event) -> String {
        match event.payload {
            EventPayload::AgentDown(_) => "Agent is down".into(),
            EventPayload::AgentInfoChanged(_) => "Agent details changed".into(),
            EventPayload::AgentInfoNew(_) => "New agent detected".into(),
            EventPayload::AgentNew(_) => "New agent detected".into(),
            EventPayload::AgentUp(_) => "Agent is up".into(),
            EventPayload::ClusterChanged(_) => "Cluster changed".into(),
            EventPayload::ClusterNew(_) => "New cluster detected".into(),
            EventPayload::NodeChanged(_) => "Datastore node details changed".into(),
            EventPayload::NodeDown(_) => "Datastore node is down".into(),
            EventPayload::NodeNew(_) => "New datastore node detected".into(),
            EventPayload::NodeUp(_) => "Datastore node is up".into(),
            EventPayload::ShardAllocationChanged(_) => "Shard status on node changed".into(),
            EventPayload::ShardAllocationNew(_) => "Shard found on node".into(),
            _ => format!("{}", event.code()),
        }
    }
}
