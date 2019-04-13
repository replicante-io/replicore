use std::io;
use std::time::Duration;

use failure::ResultExt;
use reqwest::Client as ReqwestClient;
use reqwest::RequestBuilder;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

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
    root_url: String,
}

impl Client for HttpClient {
    fn agent_info(&self) -> Result<AgentInfo> {
        let endpoint = self.endpoint("/api/unstable/info/agent");
        let request = self.client.get(&endpoint);
        self.perform(request)
    }

    fn datastore_info(&self) -> Result<DatastoreInfo> {
        let endpoint = self.endpoint("/api/unstable/info/datastore");
        let request = self.client.get(&endpoint);
        self.perform(request)
    }

    fn id(&self) -> &str {
        &self.root_url
    }

    fn shards(&self) -> Result<Shards> {
        let endpoint = self.endpoint("/api/unstable/shards");
        let request = self.client.get(&endpoint);
        self.perform(request)
    }
}

impl HttpClient {
    /// Create a new HTTP client to interact with the agent.
    pub fn make<S>(target: S, timeout: Duration) -> Result<HttpClient>
    where
        S: Into<String>,
    {
        let client = ReqwestClient::builder()
            .timeout(timeout)
            .build()
            .with_context(|_| ErrorKind::Transport("HTTP"))?;
        let target = target.into();
        let root_url = String::from(target.trim_end_matches('/'));
        Ok(HttpClient { client, root_url })
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
    fn perform<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request = request
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

    use super::HttpClient;

    #[test]
    fn enpoint_concat() {
        let client = HttpClient::make("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_root() {
        let client = HttpClient::make("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("some/path"), "proto://host:port/some/path");
    }

    #[test]
    fn enpoint_trim_path_prefix() {
        let client = HttpClient::make("proto://host:port", Duration::from_secs(15)).unwrap();
        assert_eq!(client.endpoint("/some/path"), "proto://host:port/some/path");
    }
}
