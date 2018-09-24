use clap::ArgMatches;
use reqwest::Client as ReqwestClient;

use replicante_data_models::api::Version;

use super::Result;
use super::ResultExt;


const FAIL_REQUEST_VERSION: &str = "Failed to request replicante version";


/// Replicante core HTTP API client.
pub struct Client {
    client: ReqwestClient,
    url: String,
}

impl Client {
    /// Create a new client that will connect to the given `url`.
    pub fn new<'a>(args: &ArgMatches<'a>) -> Result<Client> {
        let client = ReqwestClient::builder().build()?;
        let url = String::from(args.value_of("url").unwrap().trim_right_matches('/'));
        Ok(Client {
            client,
            url,
        })
    }

    /// Fetches the version Replicante over the API.
    pub fn version(&self) -> Result<Version> {
        let endpoint = self.endpoint("/api/v1/version");
        let request = self.client.get(&endpoint);
        let mut response = request.send().chain_err(|| FAIL_REQUEST_VERSION)?;
        let version = response.json().chain_err(|| FAIL_REQUEST_VERSION)?;
        Ok(version)
    }
}

impl Client {
    /// Utility method to build a full path for an endpoint.
    fn endpoint<S>(&self, path: S) -> String where S: Into<String> {
        let path = path.into();
        format!("{}/{}", self.url, path.trim_left_matches('/'))
    }
}
