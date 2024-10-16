//! Implementation of the API client object, to keep files organised.
use anyhow::Result;
use reqwest::Client as ReqwestClient;

use repliclient_utils::ClientOptions;

mod apply;
mod cluster_spec;
mod list;
mod naction;
mod namespace;
mod oaction;
mod platform;

/// String to set as the user agent in HTTP request.
static CLIENT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Async API client to the Replicante Control Plane.
pub struct Client {
    /// Base URL of the API server to send requests to.
    base: String,

    /// Low-level [`Client`](reqwest::Client) to perform HTTP requests with.
    client: ReqwestClient,
}

impl Client {
    /// Initialise a client with [`ClientOptions`].
    pub fn with<O>(options: O) -> Result<Client>
    where
        O: Into<ClientOptions>,
    {
        let options = options.into();
        let client = options.client(CLIENT_USER_AGENT)?;
        let client = Client {
            base: options.address,
            client: client.build()?,
        };
        Ok(client)
    }
}
