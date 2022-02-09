use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use failure::SyncFailure;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use opentracingrust::Tracer;
use reqwest::blocking::Client as ReqwestClient;
use reqwest::blocking::RequestBuilder;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::Certificate;
use reqwest::Identity;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use slog::Logger;
use uuid::Uuid;

use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::api::ActionScheduleRequest;
use replicante_models_agent::actions::ActionListItem;
use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;
use replicante_models_core::scope::Namespace;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_failure::SerializableFail;
use replicante_util_tracing::carriers::reqwest::HeadersCarrier;
use replicante_util_tracing::fail_span;

const DUPLICATE_ACTION_VARIANT: &str = "ActionAlreadyExists";

use super::Client;
use super::Error;
use super::ErrorKind;
use super::Result;

use super::metrics::CLIENT_HTTP_STATUS;
use super::metrics::CLIENT_OPS_COUNT;
use super::metrics::CLIENT_OPS_DURATION;
use super::metrics::CLIENT_OP_ERRORS_COUNT;
use super::metrics::CLIENT_TIMEOUT;

/// Interface to interact with (remote) agents over HTTP.
pub struct HttpClient {
    client: ReqwestClient,
    logger: Logger,
    root_url: String,
    tracer: Option<Arc<Tracer>>,
}

impl Client for HttpClient {
    fn action_info(&self, id: &Uuid, span: Option<SpanContext>) -> Result<ActionInfoResponse> {
        let endpoint = self.endpoint(format!("/api/unstable/actions/info/{}", id));
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let mut span = tracer
                    .span_with_options("agent.client.http.actions.info", options)
                    .auto_finish();
                span.tag("action.id", id.to_string());
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn actions_finished(&self, span: Option<SpanContext>) -> Result<Vec<ActionListItem>> {
        let endpoint = self.endpoint("/api/unstable/actions/finished");
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let span = tracer
                    .span_with_options("agent.client.http.actions.finished", options)
                    .auto_finish();
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn actions_queue(&self, span: Option<SpanContext>) -> Result<Vec<ActionListItem>> {
        let endpoint = self.endpoint("/api/unstable/actions/queue");
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let span = tracer
                    .span_with_options("agent.client.http.actions.queue", options)
                    .auto_finish();
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn agent_info(&self, span: Option<SpanContext>) -> Result<AgentInfo> {
        let endpoint = self.endpoint("/api/unstable/info/agent");
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let span = tracer
                    .span_with_options("agent.client.http.info.agent", options)
                    .auto_finish();
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn datastore_info(&self, span: Option<SpanContext>) -> Result<DatastoreInfo> {
        let endpoint = self.endpoint("/api/unstable/info/datastore");
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let span = tracer
                    .span_with_options("agent.client.http.info.datastore", options)
                    .auto_finish();
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn id(&self) -> &str {
        &self.root_url
    }

    fn shards(&self, span: Option<SpanContext>) -> Result<Shards> {
        let endpoint = self.endpoint("/api/unstable/shards");
        let request = self.client.get(&endpoint);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let span = tracer
                    .span_with_options("agent.client.http.shards", options)
                    .auto_finish();
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        self.perform(request, context).map_err(|error| match span {
            None => error,
            Some(mut span) => fail_span(error, span.as_mut()),
        })
    }

    fn schedule_action(
        &self,
        kind: &str,
        headers: &HashMap<String, String>,
        payload: ActionScheduleRequest,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let endpoint = self.endpoint(format!("/api/unstable/actions/schedule/{}", kind));
        let headers = headers
            .iter()
            .map(|(name, value)| {
                let name = HeaderName::from_lowercase(name.to_lowercase().as_bytes());
                let value = HeaderValue::from_str(value);
                (name, value)
            })
            .filter(|(name, value)| name.is_ok() && value.is_ok())
            .map(|(name, value)| {
                let name = name.unwrap();
                let value = value.unwrap();
                (name, value)
            })
            .collect();
        let request = self.client.post(&endpoint).json(&payload).headers(headers);
        let span = match (self.tracer.as_ref(), span) {
            (Some(tracer), Some(parent)) => {
                let options = StartOptions::default().child_of(parent);
                let mut span = tracer
                    .span_with_options("agent.client.http.actions.schedule", options)
                    .auto_finish();
                if let Some(id) = payload.action_id {
                    span.tag("action.id", id.to_string());
                }
                span.tag("action.kind", kind.to_string());
                Some(span)
            }
            _ => None,
        };
        let context = span.as_ref().map(|span| span.context().clone());
        // To ignore the response from the agent we need to pass a catch all type
        // or the operation will fail trying to JSON-decode the response into the unit type.
        self.perform::<serde_json::Value>(request, context)
            .map_err(|error| match span {
                None => error,
                Some(mut span) => fail_span(error, span.as_mut()),
            })?;
        Ok(())
    }
}

impl HttpClient {
    /// Create a new HTTP client to interact with the agent.
    pub fn new<S, T>(
        ns: &Namespace,
        target: S,
        timeout: Duration,
        logger: Logger,
        tracer: T,
    ) -> Result<HttpClient>
    where
        S: Into<String>,
        T: Into<Option<Arc<Tracer>>>,
    {
        let mut client = ReqwestClient::builder().timeout(timeout);

        // Set up HTTPS configuration as needed.
        client = client.use_rustls_tls();
        if let Some(ca_bundle_file) = ns.https_transport.ca_bundle.as_ref() {
            let mut ca_bundle = Vec::new();
            File::open(ca_bundle_file)
                .with_context(|_| ErrorKind::Transport("HTTP"))?
                .read_to_end(&mut ca_bundle)
                .with_context(|_| ErrorKind::Transport("HTTP"))?;
            let ca_bundle =
                Certificate::from_pem(&ca_bundle).with_context(|_| ErrorKind::Transport("HTTP"))?;
            client = client.add_root_certificate(ca_bundle);
        }
        if let Some(client_key_file) = ns.https_transport.client_key_id.as_ref() {
            let mut client_key = Vec::new();
            File::open(client_key_file)
                .with_context(|_| ErrorKind::Transport("HTTP"))?
                .read_to_end(&mut client_key)
                .with_context(|_| ErrorKind::Transport("HTTP"))?;
            let client_key =
                Identity::from_pem(&client_key).with_context(|_| ErrorKind::Transport("HTTP"))?;
            client = client.identity(client_key);
        }

        // Create reqwest::Client and the wrapping HttpClient.
        let client = client
            .build()
            .with_context(|_| ErrorKind::Transport("HTTP"))?;
        let target = target.into();
        let tracer = tracer.into();
        let root_url = String::from(target.trim_end_matches('/'));
        Ok(HttpClient {
            client,
            logger,
            root_url,
            tracer,
        })
    }
}

impl HttpClient {
    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String
    where
        S: Into<String>,
    {
        let path = path.into();
        format!("{}/{}", self.root_url, path.trim_start_matches('/'))
    }

    /// Performs a request, decoding the JSON response and tracking some stats.
    fn perform<T>(&self, request: RequestBuilder, span: Option<SpanContext>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let headers = {
            let mut headers = HeaderMap::new();
            if let (Some(tracer), Some(span)) = (self.tracer.as_ref(), span) {
                if let Err(error) = HeadersCarrier::inject(&span, &mut headers, tracer) {
                    let error = SyncFailure::new(error);
                    capture_fail!(
                        &error,
                        self.logger,
                        "Failed to inject tracing context into headers";
                        failure_info(&error),
                    );
                }
            }
            headers
        };
        let request = request
            .headers(headers)
            .build()
            .with_context(|_| ErrorKind::Transport("HTTP"))?;
        let endpoint = String::from(&request.url().as_str()[self.root_url.len()..]);
        CLIENT_OPS_COUNT.with_label_values(&[&endpoint]).inc();
        let timer = CLIENT_OPS_DURATION
            .with_label_values(&[&endpoint])
            .start_timer();
        let response = self
            .client
            .execute(request)
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                if error.is_timeout() {
                    CLIENT_TIMEOUT.with_label_values(&[&endpoint]).inc();
                }
                error
            })
            .with_context(|_| ErrorKind::Transport("HTTP"))?;
        timer.observe_duration();
        let status = response.status();
        CLIENT_HTTP_STATUS
            .with_label_values(&[&endpoint, status.as_str()])
            .inc();
        let result = match status {
            StatusCode::CONFLICT => response
                .json::<SerializableFail>()
                .with_context(|_| ErrorKind::JsonDecode)
                .map_err(Error::from)
                .and_then(|response| match response.variant {
                    Some(ref message) if message == DUPLICATE_ACTION_VARIANT => {
                        Err(ErrorKind::DuplicateAction.into())
                    }
                    _ => Err(ErrorKind::Remote(response.error).into()),
                }),
            status if status < StatusCode::BAD_REQUEST => response
                .json()
                .with_context(|_| ErrorKind::JsonDecode)
                .map_err(Error::from),
            _ => response
                .json::<SerializableFail>()
                .with_context(|_| ErrorKind::JsonDecode)
                .map_err(Error::from)
                .and_then(|response| Err(ErrorKind::Remote(response.error).into())),
        };
        result.map_err(|error| {
            CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
            error
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use replicante_models_core::scope::Namespace;
    use replicante_models_core::scope::NsHttpsTransport;

    use super::HttpClient;

    #[test]
    fn enpoint_concat() {
        let ns = Namespace {
            ns_id: "test".into(),
            https_transport: NsHttpsTransport::default(),
        };
        let logger = Logger::root(Discard, o!());
        let client = HttpClient::new(
            &ns,
            "proto://host:port",
            Duration::from_secs(15),
            logger,
            None,
        )
        .unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_root() {
        let ns = Namespace {
            ns_id: "test".into(),
            https_transport: NsHttpsTransport::default(),
        };
        let logger = Logger::root(Discard, o!());
        let client = HttpClient::new(
            &ns,
            "proto://host:port",
            Duration::from_secs(15),
            logger,
            None,
        )
        .unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_path_prefix() {
        let ns = Namespace {
            ns_id: "test".into(),
            https_transport: NsHttpsTransport::default(),
        };
        let logger = Logger::root(Discard, o!());
        let client = HttpClient::new(
            &ns,
            "proto://host:port",
            Duration::from_secs(15),
            logger,
            None,
        )
        .unwrap();
        assert_eq!(client.endpoint("/some/path"), "proto://host:port/some/path");
    }
}
