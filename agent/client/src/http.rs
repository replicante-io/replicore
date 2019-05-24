use std::io;
use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use failure::SyncFailure;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use reqwest::header::HeaderMap;
use reqwest::Client as ReqwestClient;
use reqwest::RequestBuilder;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use slog::Logger;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_tracing::carriers::reqwest::HeadersCarrier;

use super::Client;
use super::Error;
use super::ErrorKind;
use super::Result;

use super::metrics::CLIENT_HTTP_STATUS;
use super::metrics::CLIENT_OPS_COUNT;
use super::metrics::CLIENT_OPS_DURATION;
use super::metrics::CLIENT_OP_ERRORS_COUNT;
use super::metrics::CLIENT_TIMEOUT;

/// Decode useful portions of error messages from clients.
#[derive(Deserialize)]
struct ClientError {
    error: String,
}

/// Interface to interact with (remote) agents over HTTP.
pub struct HttpClient {
    client: ReqwestClient,
    logger: Logger,
    root_url: String,
    tracer: Option<Arc<Tracer>>,
}

impl Client for HttpClient {
    fn agent_info(&self, span: Option<SpanContext>) -> Result<AgentInfo> {
        let endpoint = self.endpoint("/api/unstable/info/agent");
        let request = self.client.get(&endpoint);
        self.perform(request, span)
    }

    fn datastore_info(&self, span: Option<SpanContext>) -> Result<DatastoreInfo> {
        let endpoint = self.endpoint("/api/unstable/info/datastore");
        let request = self.client.get(&endpoint);
        self.perform(request, span)
    }

    fn id(&self) -> &str {
        &self.root_url
    }

    fn shards(&self, span: Option<SpanContext>) -> Result<Shards> {
        let endpoint = self.endpoint("/api/unstable/shards");
        let request = self.client.get(&endpoint);
        self.perform(request, span)
    }
}

impl HttpClient {
    /// Create a new HTTP client to interact with the agent.
    pub fn make<S, T>(target: S, timeout: Duration, logger: Logger, tracer: T) -> Result<HttpClient>
    where
        S: Into<String>,
        T: Into<Option<Arc<Tracer>>>,
    {
        let client = ReqwestClient::builder()
            .timeout(timeout)
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
        let mut response = self
            .client
            .execute(request)
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                // Look at the inner error, if any, to check if it is a timout.
                let inner_kind = error
                    .get_ref()
                    .and_then(|error| error.downcast_ref::<io::Error>())
                    .map(io::Error::kind);
                match inner_kind {
                    Some(io::ErrorKind::TimedOut) | Some(io::ErrorKind::WouldBlock) => {
                        CLIENT_TIMEOUT.with_label_values(&[&endpoint]).inc();
                    }
                    _ => (),
                };
                error
            })
            .with_context(|_| ErrorKind::Transport("HTTP"))?;
        timer.observe_duration();
        let status = response.status();
        CLIENT_HTTP_STATUS
            .with_label_values(&[&endpoint, status.as_str()])
            .inc();
        if response.status() < StatusCode::BAD_REQUEST {
            response
                .json()
                .with_context(|_| ErrorKind::JsonDecode)
                .map_err(Error::from)
                .map_err(|error| {
                    CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                    error
                })
        } else {
            response
                .json::<ClientError>()
                .with_context(|_| ErrorKind::JsonDecode)
                .map_err(Error::from)
                .and_then(|response| Err(ErrorKind::Remote(response.error).into()))
                .map_err(|error| {
                    CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                    error
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use super::HttpClient;

    #[test]
    fn enpoint_concat() {
        let logger = Logger::root(Discard, o!());
        let client =
            HttpClient::make("proto://host:port", Duration::from_secs(15), logger, None).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_root() {
        let logger = Logger::root(Discard, o!());
        let client =
            HttpClient::make("proto://host:port", Duration::from_secs(15), logger, None).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_path_prefix() {
        let logger = Logger::root(Discard, o!());
        let client =
            HttpClient::make("proto://host:port", Duration::from_secs(15), logger, None).unwrap();
        assert_eq!(client.endpoint("/some/path"), "proto://host:port/some/path");
    }
}
