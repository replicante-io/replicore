use anyhow::Context as _;
use anyhow::Result;
use reqwest::Certificate;
use reqwest::Client;
use reqwest::Identity;
use reqwest::RequestBuilder;
use reqwest::StatusCode;
use slog::debug;
use slog::Logger;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use replicante_util_failure::SerializableFail;

use crate::context::Context;

/// Abstraction over an async reqwest client to hide the low-level operations.
pub struct HttpClient {
    client: Client,
    url: String,
}

impl HttpClient {
    /// Initialise the low-level HTTP client.
    pub async fn new(logger: &Logger, context: &Context) -> Result<HttpClient> {
        let connection = &context.connection;
        let mut client = Client::builder().use_rustls_tls();
        if let Some(ca_path) = &connection.ca_bundle {
            debug!(logger, "Adding configured CA bundle"; "path" => ca_path);
            let mut ca_bundle = Vec::new();
            File::open(ca_path)
                .await
                .with_context(|| format!("Failed to open CA bundle at {}", ca_path))?
                .read_to_end(&mut ca_bundle)
                .await
                .with_context(|| format!("Failed to read CA bundle from {}", ca_path))?;
            let ca_bundle = Certificate::from_pem(&ca_bundle)
                .with_context(|| format!("Failed to decode CA bundle from {}", ca_path))?;
            client = client.add_root_certificate(ca_bundle);
        }
        if let Some(key_path) = &connection.client_key {
            debug!(logger, "Adding configured client key"; "path" => key_path);
            let mut client_key = Vec::new();
            File::open(key_path)
                .await
                .with_context(|| format!("Failed to open client key at {}", key_path))?
                .read_to_end(&mut client_key)
                .await
                .with_context(|| format!("Failed to read client key from {}", key_path))?;
            let client_key = Identity::from_pem(&client_key)
                .with_context(|| format!("Failed to decode client key from {}", key_path))?;
            client = client.identity(client_key);
        }
        let client = client
            .build()
            .with_context(|| "Failed to initialise the HTTP client")?;
        let url = connection.url.trim_end_matches('/').to_string();
        Ok(HttpClient { client, url })
    }

    /// Start a DELETE request to the API server.
    pub fn delete(&self, uri: &str) -> RequestBuilder {
        let url = format!("{}{}", self.url, uri);
        self.client.delete(&url)
    }

    /// Start a GET request to the API server.
    pub fn get(&self, uri: &str) -> RequestBuilder {
        let url = format!("{}{}", self.url, uri);
        self.client.get(&url)
    }

    /// Start a POST request to the API server.
    pub fn post(&self, uri: &str) -> RequestBuilder {
        let url = format!("{}{}", self.url, uri);
        self.client.post(&url)
    }

    /// Send a request to the API server.
    pub async fn send(&self, request: RequestBuilder) -> Result<Response> {
        let response = request
            .send()
            .await
            .with_context(|| "Failed to send API request")?;
        let response = Response::build(response).await?;
        Ok(response)
    }
}

/// Response instance to store status code and JSON body.
pub struct Response {
    body: serde_json::Value,
    status: StatusCode,
}

impl Response {
    /// JSON-decode the response and add some utility methods.
    async fn build(response: reqwest::Response) -> Result<Response> {
        let status = response.status();
        let body = response
            .json()
            .await
            .with_context(|| "Failed to JSON decode API response")?;
        Ok(Response { body, status })
    }

    /// Decode the body of the response to the given type.
    pub fn body_as<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let body = self.body.clone();
        let body = serde_json::from_value(body)?;
        Ok(body)
    }

    /// Check the HTTP response status code for common errors.
    pub fn check_status(&self) -> Result<()> {
        match self.status {
            // Missing resources or authentication errors.
            StatusCode::NOT_FOUND => anyhow::bail!(super::ApiNotFound),

            // Status < 400 indicate success of the operation.
            status if status.as_u16() < 400 => Ok(()),

            // Other remote errors.
            _ => {
                let body = self.body.clone();
                let remote: SerializableFail = serde_json::from_value(body)
                    .with_context(|| "unable to JSON decode API response")?;
                let mut error: Option<anyhow::Error> = None;
                for layer in remote.layers.into_iter().rev() {
                    let layer = format!("(remote) {}", layer);
                    let err = match error {
                        None => anyhow::anyhow!(layer),
                        Some(error) => error.context(layer),
                    };
                    error = Some(err);
                }
                match error {
                    None => anyhow::bail!(remote.error),
                    Some(error) => anyhow::bail!(error),
                }
            }
        }
    }

    /// Extract the response body and discards additional metadata.
    pub fn into_body(self) -> serde_json::Value {
        self.body
    }

    /// Access the response status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }
}
