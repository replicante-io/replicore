use clap::ArgMatches;
use failure::ResultExt;
use reqwest::Client as ReqwestClient;

use replicante_models_core::api::Version;

use super::ErrorKind;
use super::Result;

const ENDPOINT_VERSION: &str = "/api/unstable/introspect/version";

/// Replicante core HTTP API client.
pub struct Client {
    client: ReqwestClient,
    url: String,
}

impl Client {
    /// Create a new client that will connect to the given `url`.
    pub fn new<'a>(args: &ArgMatches<'a>) -> Result<Client> {
        let client = ReqwestClient::builder()
            .build()
            .with_context(|_| ErrorKind::HttpClient)?;
        let url = String::from(args.value_of("url").unwrap().trim_end_matches('/'));
        Ok(Client { client, url })
    }

    /// Fetches the version Replicante over the API.
    pub fn version(&self) -> Result<Version> {
        let endpoint = self.endpoint(ENDPOINT_VERSION);
        let request = self.client.get(&endpoint);
        let mut response = request
            .send()
            .with_context(|_| ErrorKind::ReplicanteRequest(ENDPOINT_VERSION))?;
        let version = response
            .json()
            .with_context(|_| ErrorKind::ReplicanteJsonDecode)?;
        Ok(version)
    }
}

impl Client {
    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String
    where
        S: Into<String>,
    {
        let path = path.into();
        format!("{}/{}", self.url, path.trim_start_matches('/'))
    }
}
