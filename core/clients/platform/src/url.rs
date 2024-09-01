//! URL addressed platform client factories.
//!
//! Using URLs to address client provides users with a familiar configuration
//! while giving us the flexibility to support many different implementations.
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::platform::PlatformTransportUrl;

use repliclient_utils::ClientOptions;
use replicore_context::Context;
use repliplatform_client::Client;
use repliplatform_client_http::HttpClient;

/// Convenience type for heap allocated [`UrlFactory`]s.
pub type ArcedUrlFactory = Arc<dyn UrlFactory>;

/// Async function to initialise Platform clients on demand.
#[async_trait::async_trait]
pub trait UrlFactory: Send + Sync {
    /// Initialise a new [`Platform`] client.
    async fn init(&self, context: &Context, transport: &PlatformTransportUrl) -> Result<Client>;
}

/// HTTP(S) clients factory.
pub struct HttpClientFactory;

#[async_trait::async_trait]
impl UrlFactory for HttpClientFactory {
    async fn init(&self, _: &Context, platform: &PlatformTransportUrl) -> Result<Client> {
        let options = ClientOptions::url(&platform.base_url);
        // TODO: TLS CA Bundle support.
        let client = HttpClient::with(options)?;
        Ok(Client::from(client))
    }
}
