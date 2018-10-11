use std::io;
use std::time::Duration;

use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use reqwest::Client as ReqwestClient;
use reqwest::RequestBuilder;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use slog::Logger;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::Client;
use super::Result;
use super::ResultExt;


static FAIL_INFO_FETCH: &'static str = "Failed to fetch agent info";
static FAIL_STATUS_FETCH: &'static str = "Failed to fetch agent status";


lazy_static! {
    /// Counter of HTTP response statused.
    static ref CLIENT_HTTP_STATUS: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_http_status", "Number of HTTP status for an endpoint"),
        &["endpoint", "status"]
    ).expect("Failed to create CLIENT_HTTP_STATUS counter");

    /// Counter for agent operation errors.
    static ref CLIENT_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_operation_errors", "Number of agent operations failed"),
        &["endpoint"]
    ).expect("Failed to create CLIENT_OP_ERRORS_COUNT counter");

    /// Counter for agent operations.
    static ref CLIENT_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_operations", "Number of agent operations issued"),
        &["endpoint"]
    ).expect("Failed to create CLIENT_OPS_COUNT counter");

    /// Observe duration of agent operations.
    static ref CLIENT_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicante_agentclient_operations_duration",
            "Duration (in seconds) of agent operations"
        ),
        &["endpoint"]
    ).expect("Failed to create CLIENT_OPS_DURATION histogram");

    /// Counter for agent operation timeout errors.
    static ref CLIENT_TIMEOUT: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_timeout", "Number of agent operations that timed out"),
        &["endpoint"]
    ).expect("Failed to create CLIENT_TIMEOUT counter");
}


/// Decode useful portions of error messages from clients.
#[derive(Deserialize)]
struct ClientError {
    error: String,
}


/// Interface to interact with (remote) agents over HTTP.
pub struct HttpClient {
    client: ReqwestClient,
    root_url: String,
}

impl Client for HttpClient {
    fn agent_info(&self) -> Result<AgentInfo> {
        let endpoint = self.endpoint("/api/v1/info/agent");
        let request = self.client.get(&endpoint);
        self.perform(request)
            .chain_err(|| FAIL_INFO_FETCH)
    }

    fn datastore_info(&self) -> Result<DatastoreInfo> {
        let endpoint = self.endpoint("/api/v1/info/datastore");
        let request = self.client.get(&endpoint);
        self.perform(request)
            .chain_err(|| FAIL_INFO_FETCH)
    }

    fn shards(&self) -> Result<Shards> {
        let endpoint = self.endpoint("/api/v1/shards");
        let request = self.client.get(&endpoint);
        self.perform(request)
            .chain_err(|| FAIL_STATUS_FETCH)
    }
}

impl HttpClient {
    /// Creates a new HTTP client to interact with the agent.
    pub fn new<S>(target: S, timeout: Duration) -> Result<HttpClient>
        where S: Into<String>,
    {
        let client = ReqwestClient::builder().timeout(timeout).build()?;
        let target = target.into();
        let root_url = String::from(target.trim_right_matches('/'));
        Ok(HttpClient {
            client,
            root_url,
        })
    }

    /// Attemps to register metrics with the Registry.
    ///
    /// Metrics that fail to register are logged and ignored.
    ///
    /// **This method should be called before using any client**.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        if let Err(err) = registry.register(Box::new(CLIENT_HTTP_STATUS.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_HTTP_STATUS"; "error" => error);
        }
        if let Err(err) = registry.register(Box::new(CLIENT_OP_ERRORS_COUNT.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_OP_ERRORS_COUNT"; "error" => error);
        }
        if let Err(err) = registry.register(Box::new(CLIENT_OPS_COUNT.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_OPS_COUNT"; "error" => error);
        }
        if let Err(err) = registry.register(Box::new(CLIENT_OPS_DURATION.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_OPS_DURATION"; "error" => error);
        }
        if let Err(err) = registry.register(Box::new(CLIENT_TIMEOUT.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_TIMEOUT"; "error" => error);
        }
    }
}

impl HttpClient {
    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String 
        where S: Into<String>,
    {
        let path = path.into();
        format!("{}/{}", self.root_url, path.trim_left_matches('/'))
    }

    /// Performs a request, decoding the JSON response and tracking some stats.
    fn perform<T>(&self, request: RequestBuilder) -> Result<T>
        where T: DeserializeOwned
    {
        let request = request.build()?;
        let endpoint = String::from(&request.url().as_str()[self.root_url.len()..]);
        CLIENT_OPS_COUNT.with_label_values(&[&endpoint]).inc();
        let timer = CLIENT_OPS_DURATION.with_label_values(&[&endpoint]).start_timer();
        let mut response = self.client.execute(request)
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                // Look at the inner error, if any, to check if it is a timout.
                let inner_kind = error.get_ref()
                    .and_then(|error| error.downcast_ref::<io::Error>())
                    .map(|error| error.kind());
                match inner_kind {
                    Some(io::ErrorKind::TimedOut) |
                    Some(io::ErrorKind:: WouldBlock) => {
                        CLIENT_TIMEOUT.with_label_values(&[&endpoint]).inc();
                    }
                    _ => (),
                };
                error
            })?;
        timer.observe_duration();
        let status = response.status();
        CLIENT_HTTP_STATUS.with_label_values(&[&endpoint, status.as_str()]).inc();
        if response.status() < StatusCode::BAD_REQUEST {
            response.json().map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&[&endpoint]).inc();
                error.into()
            })
        } else {
            response.json::<ClientError>()
                .map_err(|error| error.into())
                .and_then(|response| Err(response.error.into()))
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
    use super::HttpClient;

    #[test]
    fn enpoint_concat() {
        let client = HttpClient::new("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_root() {
        let client = HttpClient::new("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_path_prefix() {
        let client = HttpClient::new("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("/some/path"), "proto://host:port/some/path");
    }
}
