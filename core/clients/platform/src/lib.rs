//! Registry and factories to initialise Platform API clients on demand.
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformTransport;
use replisdk::core::models::platform::PlatformTransportUrl;

use replicore_context::Context;
use repliplatform_client::Client;

mod url;

pub mod errors;
pub use self::url::HttpClientFactory;
pub use self::url::UrlFactory;

use self::url::ArcedUrlFactory;

/// Registry of Platform API client factories.
#[derive(Clone)]
pub struct PlatformClients {
    url_schemas: HashMap<String, ArcedUrlFactory>,
}

impl PlatformClients {
    /// Create a [`PlatformClients`] registry with no factories configured.
    pub fn empty() -> PlatformClients {
        PlatformClients {
            url_schemas: Default::default(),
        }
    }

    /// Initialise a client to interact with a [`Platform`].
    pub async fn factory(&self, context: &Context, platform: &Platform) -> Result<Client> {
        match &platform.transport {
            PlatformTransport::Url(transport) => {
                self.url_factory(context, platform, transport).await
            }
        }
    }

    /// Register a URL client factory for a schema.
    pub fn with_url_factory<F, S>(&mut self, schema: S, factory: F) -> &mut Self
    where
        S: Into<String>,
        F: UrlFactory + 'static,
    {
        let schema = schema.into();
        let factory = Arc::new(factory);
        self.url_schemas.insert(schema, factory);
        self
    }
}

impl PlatformClients {
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
                let error = self::errors::UrlClientNoSchema {
                    ns_id: platform.ns_id.clone(),
                    name: platform.name.clone(),
                };
                anyhow::bail!(error);
            }
        };
        let factory = match self.url_schemas.get(schema) {
            Some(factory) => factory,
            None => {
                let error = self::errors::UrlClientUnknownSchema {
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

impl Default for PlatformClients {
    fn default() -> Self {
        PlatformClients::empty()
    }
}
