use std::fs::File;
use std::io::Read;
use std::time::Duration;

use failure::ResultExt;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Map;
use serde_json::Value;

use replicante_models_core::ClusterDiscovery;

use crate::config::HttpConfig;
use crate::config::HttpRequestMethod;
use crate::metrics::DISCOVERY_ERRORS;
use crate::metrics::DISCOVERY_TOTAL;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Response expected from an HTTP discovery server.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct DiscoveryResponse {
    pub clusters: Vec<ClusterDiscovery>,
    pub cursor: Option<String>,
}

/// HTTP discovery iterator.
///
/// Calls to `Iter::next` will perform HTTP(S) POST requests against a remote
/// server to fetch cluster discovery records to return to the user.
/// Response pagination is handled transparently as part of this process.
///
/// ## Writing a server
/// TODO: request format (POST + JSON body)
/// TODO: response format (clusters + cursor)
pub struct Iter {
    body: Map<String, Value>,
    buffer: Vec<ClusterDiscovery>,
    client: Option<Client>,
    config: Option<HttpConfig>,
    cursor: Option<String>,
    failed_or_done: bool,
    method: HttpRequestMethod,
    url: String,
}

impl Iter {
    pub fn new(config: HttpConfig) -> Iter {
        let body = config.body.clone().unwrap_or_default();
        let method = config.method.clone();
        let url = config.url.clone();
        Iter {
            body,
            buffer: Vec::new(),
            client: None,
            config: Some(config),
            cursor: None,
            failed_or_done: false,
            method,
            url,
        }
    }

    /// Ensure an HTTP client is available for requests, creating one if needed.
    fn ensure_client(&mut self) -> Option<Result<()>> {
        if self.client.is_none() {
            let client = match self.config.take() {
                None => return None,
                Some(config) => match Iter::init_client(config) {
                    Ok(client) => client,
                    Err(error) => {
                        DISCOVERY_ERRORS.with_label_values(&["http"]).inc();
                        self.failed_or_done = true;
                        return Some(Err(error));
                    }
                },
            };
            self.client = Some(client);
        }
        Some(Ok(()))
    }

    /// Initialise the HTTP client to make requests with.
    fn init_client(config: HttpConfig) -> Result<Client> {
        let mut headers = HeaderMap::with_capacity(config.headers.len());
        for (key, value) in config.headers {
            let key = HeaderName::from_bytes(key.as_bytes())
                .with_context(|_| ErrorKind::HttpHeaderName(key.to_string()))?;
            let value = HeaderValue::from_str(&value)
                .with_context(|_| ErrorKind::HttpHeaderValue(value.to_string()))?;
            headers.insert(key, value);
        }
        let mut builder = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.timeout));
        if let Some(ca_cert) = config.tls.ca_cert {
            let mut buf = Vec::new();
            File::open(ca_cert)
                .with_context(|_| ErrorKind::HttpCertLoad)?
                .read_to_end(&mut buf)
                .with_context(|_| ErrorKind::HttpCertLoad)?;
            let cert =
                reqwest::Certificate::from_pem(&buf).with_context(|_| ErrorKind::HttpCertLoad)?;
            builder = builder.add_root_certificate(cert);
        }
        if let Some(client_cert) = config.tls.client_cert {
            let mut buf = Vec::new();
            File::open(client_cert)
                .with_context(|_| ErrorKind::HttpCertLoad)?
                .read_to_end(&mut buf)
                .with_context(|_| ErrorKind::HttpCertLoad)?;
            let id = reqwest::Identity::from_pem(&buf).with_context(|_| ErrorKind::HttpCertLoad)?;
            builder = builder.identity(id);
        }
        let client = builder.build().with_context(|_| ErrorKind::HttpClient)?;
        Ok(client)
    }

    /// Request a new set of discoveries from the remote HTTP server.
    fn request_more(&mut self) -> Option<Result<DiscoveryResponse>> {
        let client = match self.ensure_client() {
            None => return None,
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(())) => self.client.as_ref().expect("client not initialised"),
        };
        let response = match &self.method {
            HttpRequestMethod::Get => client.get(&self.url),
            HttpRequestMethod::Post => {
                let mut body = self.body.clone();
                let cursor = match self.cursor.clone() {
                    None => Value::Null,
                    Some(cursor) => Value::String(cursor),
                };
                body.insert("cursor".into(), cursor);
                client.post(&self.url).json(&body)
            }
        };
        let response = response
            .send()
            .with_context(|_| ErrorKind::HttpRequest)
            .map_err(Error::from);
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                DISCOVERY_ERRORS.with_label_values(&["http"]).inc();
                self.failed_or_done = true;
                return Some(Err(error));
            }
        };
        let mut response = match response.error_for_status() {
            Ok(response) => response,
            Err(error) => {
                DISCOVERY_ERRORS.with_label_values(&["http"]).inc();
                let error = Err(error)
                    .with_context(|_| ErrorKind::HttpRequest)
                    .map_err(Error::from);
                self.failed_or_done = true;
                return Some(error);
            }
        };
        let response: DiscoveryResponse = match response.json() {
            Ok(response) => response,
            Err(error) => {
                DISCOVERY_ERRORS.with_label_values(&["http"]).inc();
                let error = Err(error)
                    .with_context(|_| ErrorKind::HttpRequest)
                    .map_err(Error::from);
                self.failed_or_done = true;
                return Some(error);
            }
        };
        Some(Ok(response))
    }
}

impl Iterator for Iter {
    type Item = Result<ClusterDiscovery>;
    fn next(&mut self) -> Option<Self::Item> {
        DISCOVERY_TOTAL.with_label_values(&["http"]).inc();
        // Return any buffered discoveries.
        if let Some(cluster) = self.buffer.pop() {
            return Some(Ok(cluster));
        }

        // Stop trying once we enter a failed state or there are no more discoveries to return.
        if self.failed_or_done {
            return None;
        }

        // Request more discoveries to return.
        let response = match self.request_more() {
            None => return None,
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(response)) => response,
        };
        self.buffer = response.clusters;
        self.buffer.reverse();

        // Update the state of the remote cursor to support pagination.
        self.cursor = response.cursor;
        if self.cursor.is_none() {
            self.failed_or_done = true;
        }

        // Return the top of the buffer.
        self.buffer.pop().map(Ok)
    }
}
