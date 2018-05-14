use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramVec;
use prometheus::Opts;
use prometheus::Registry;

use reqwest::Client as ReqwestClient;
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
    /// Counter for agent operations.
    static ref CLIENT_OPS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_operations", "Number of agent operations issued"),
        &["endpoint"]
    ).expect("Failed to create replicante_agentclient_operations counter");

    /// Counter for agent operation errors.
    static ref CLIENT_OP_ERRORS_COUNT: CounterVec = CounterVec::new(
        Opts::new("replicante_agentclient_operation_errors", "Number of agent operations failed"),
        &["endpoint"]
    ).expect("Failed to create replicante_agentclient_operation_errors counter");

    /// Observe duration of agent operations.
    static ref CLIENT_OPS_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "replicante_agentclient_operations_duration",
            "Duration (in seconds) of agent operations"
        ),
        &["endpoint"]
    ).expect("Failed to create CLIENT_OPS_DURATION histogram");
}




/// Interface to interact with (remote) agents over HTTP.
pub struct HttpClient {
    client: ReqwestClient,
    root_url: String,
}

impl Client for HttpClient {
    fn agent_info(&self) -> Result<AgentInfo> {
        CLIENT_OPS_COUNT.with_label_values(&["/api/v1/info/agent"]).inc();
        let _timer = CLIENT_OPS_DURATION.with_label_values(&["/api/v1/info/agent"]).start_timer();
        let endpoint = self.endpoint("/api/v1/info/agent");
        let mut request = self.client.get(&endpoint);
        let mut response = request.send()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/info/agent"]).inc();
                error
            })
            .chain_err(|| FAIL_INFO_FETCH)?;
        let info = response.json()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/info/agent"]).inc();
                error
            })
            .chain_err(|| FAIL_INFO_FETCH)?;
        Ok(info)
    }

    fn datastore_info(&self) -> Result<DatastoreInfo> {
        CLIENT_OPS_COUNT.with_label_values(&["/api/v1/info/datastore"]).inc();
        let _timer = CLIENT_OPS_DURATION.with_label_values(&["/api/v1/info/datastore"]).start_timer();
        let endpoint = self.endpoint("/api/v1/info/datastore");
        let mut request = self.client.get(&endpoint);
        let mut response = request.send()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/info/datastore"]).inc();
                error
            })
            .chain_err(|| FAIL_INFO_FETCH)?;
        let info = response.json()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/info/datastore"]).inc();
                error
            })
            .chain_err(|| FAIL_INFO_FETCH)?;
        Ok(info)
    }

    fn shards(&self) -> Result<Shards> {
        CLIENT_OPS_COUNT.with_label_values(&["/api/v1/shards"]).inc();
        let _timer = CLIENT_OPS_DURATION.with_label_values(&["/api/v1/shards"]).start_timer();
        let endpoint = self.endpoint("/api/v1/shards");
        let mut request = self.client.get(&endpoint);
        let mut response = request.send()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/shards"]).inc();
                error
            })
            .chain_err(|| FAIL_STATUS_FETCH)?;
        let shards = response.json()
            .map_err(|error| {
                CLIENT_OP_ERRORS_COUNT.with_label_values(&["/api/v1/shards"]).inc();
                error
            })
            .chain_err(|| FAIL_STATUS_FETCH)?;
        Ok(shards)
    }
}

impl HttpClient {
    /// Creates a new HTTP client to interact with the agent.
    pub fn new<S>(target: S) -> Result<HttpClient>
        where S: Into<String>,
    {
        let client = ReqwestClient::builder().build()?;
        let target = target.into();
        let root_url = String::from(target.trim_right_matches('/'));
        Ok(HttpClient {
            client,
            root_url,
        })
    }

    /// Attemps to register metrics with the Repositoy.
    ///
    /// Metrics that fail to register are logged and ignored.
    ///
    /// **This method should be called before using any client**.
    pub fn register_metrics(logger: &Logger, registry: &Registry) {
        if let Err(err) = registry.register(Box::new(CLIENT_OPS_COUNT.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_OPS_COUNT"; "error" => error);
        }
        if let Err(err) = registry.register(Box::new(CLIENT_OPS_DURATION.clone())) {
            let error = format!("{:?}", err);
            debug!(logger, "Failed to register CLIENT_OPS_DURATION"; "error" => error);
        }
    }

    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String 
        where S: Into<String>,
    {
        let path = path.into();
        format!("{}/{}", self.root_url, path.trim_left_matches('/'))
    }
}


#[cfg(test)]
mod tests {
    use super::HttpClient;

    #[test]
    fn enpoint_concat() {
        let client = HttpClient::new("proto://host:port").unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_root() {
        let client = HttpClient::new("proto://host:port/").unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_path_prefix() {
        let client = HttpClient::new("proto://host:port/").unwrap();
        assert_eq!(client.endpoint("/some/path"), "proto://host:port/some/path");
    }
}
