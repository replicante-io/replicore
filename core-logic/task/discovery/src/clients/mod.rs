//! Factory for Platform clients used during discovery tasks.
use std::collections::HashMap;

use anyhow::Result;

use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformTransport;
use replisdk::core::models::platform::PlatformTransportUrl;

use replicore_context::Context;
use repliplatform_client::Client;

mod http;

pub use self::http::HttpClientFactory;

/// Registry of [`Platform`] client factories.
#[derive(Default)]
pub struct Clients {
    url_schemas: HashMap<String, Box<dyn UrlClientFactory>>,
}

impl Clients {
    /// Initialise a client to interact with a [`Platform`].
    pub async fn factory(&self, context: &Context, platform: &Platform) -> Result<Client> {
        match &platform.transport {
            PlatformTransport::Url(transport) => {
                self.url_factory(context, platform, transport).await
            }
        }
    }

    /// Initialise a client to connect using a supported URL schema.
    async fn url_factory(
        &self,
        context: &Context,
        platform: &Platform,
        transport: &PlatformTransportUrl,
    ) -> Result<Client> {
        let (schema, _) = match transport.base_url.split_once(':') {
            Some(parts) => parts,
            None => {
                let error = crate::errors::UrlClientNoSchema {
                    ns_id: platform.ns_id.clone(),
                    name: platform.name.clone(),
                };
                anyhow::bail!(error);
            }
        };
        let factory = match self.url_schemas.get(schema) {
            Some(factory) => factory,
            None => {
                let error = crate::errors::UrlClientUnknownSchema {
                    ns_id: platform.ns_id.clone(),
                    name: platform.name.clone(),
                    schema: schema.to_string(),
                };
                anyhow::bail!(error);
            }
        };
        factory.init(context, transport).await
    }
}

impl Clients {
    /// Register a URL client factory for a schema.
    #[allow(dead_code)]
    pub fn with_url_factory<F, S>(&mut self, schema: S, factory: F) -> &mut Self
    where
        S: Into<String>,
        F: UrlClientFactory + 'static,
    {
        let schema = schema.into();
        let factory = Box::new(factory);
        self.url_schemas.insert(schema, factory);
        self
    }
}

/// Async function to initialise [`Platform`] clients on demand.
#[async_trait::async_trait]
pub trait UrlClientFactory: Send + Sync {
    /// Initialise a new [`Platform`] client.
    async fn init(&self, context: &Context, transport: &PlatformTransportUrl) -> Result<Client>;
}
