use clap::ArgMatches;
use failure::ResultExt;
use reqwest::Client as ReqwestClient;

use replicante_data_models::api::Version;

use super::ErrorKind;
use super::Result;


/// Replicante core HTTP API client.
pub struct Client {
    client: ReqwestClient,
    url: String,
}

impl Client {
    /// Create a new client that will connect to the given `url`.
    pub fn new<'a>(args: &ArgMatches<'a>) -> Result<Client> {
        let client = ReqwestClient::builder().build().with_context(|_| ErrorKind::HttpClient)?;
        let url = String::from(args.value_of("url").unwrap().trim_end_matches('/'));
        Ok(Client {
            client,
            url,
        })
    }

    /// Fetches the version Replicante over the API.
    pub fn version(&self) -> Result<Version> {
        let endpoint = self.endpoint("/api/v1/version");
        let request = self.client.get(&endpoint);
        let mut response = request.send()
            .with_context(|_| ErrorKind::ReplicanteRequest("/api/v1/version"))?;
        let version = response.json()
            .with_context(|_| ErrorKind::ReplicanteJsonDecode)?;
        Ok(version)
    }
}

impl Client {
    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String where S: Into<String> {
        let path = path.into();
        format!("{}/{}", self.url, path.trim_start_matches('/'))
    }
}
