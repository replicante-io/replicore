use reqwest::Client as ReqwestClient;

use replicante_agent_models::AgentInfo;

use super::Client;
use super::Result;
use super::ResultExt;


static FAIL_INFO_FETCH: &'static str = "Failed to fetch agent info";


/// Interface to interact with (remote) agents over HTTP.
pub struct HttpClient {
    client: ReqwestClient,
    root_url: String,
}

impl Client for HttpClient {
    fn info(&self) -> Result<AgentInfo> {
        let endpoint = self.endpoint("/api/v1/info");
        let mut request = self.client.get(&endpoint);
        let mut response = request.send().chain_err(|| FAIL_INFO_FETCH)?;
        let info = response.json().chain_err(|| FAIL_INFO_FETCH)?;
        Ok(info)
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
