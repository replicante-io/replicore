//! HTTP(S) clients factory.
use anyhow::Result;

use replisdk::core::models::platform::PlatformTransportHttp;

use replicore_context::Context;
use repliplatform_client::Client;
use repliplatform_client_http::ClientOptions;
use repliplatform_client_http::HttpClient;

use super::UrlClientFactory;

/// HTTP(S) clients factory.
pub struct HttpClientFactory;

#[async_trait::async_trait]
impl UrlClientFactory for HttpClientFactory {
    async fn init(&self, _: &Context, platform: &PlatformTransportHttp) -> Result<Client> {
        let options = ClientOptions::url(&platform.base_url);
        let client = HttpClient::with(options)?;
        Ok(Client::from(client))
    }
}
