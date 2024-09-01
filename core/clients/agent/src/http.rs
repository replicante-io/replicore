//! HTTP(S) clients factory.
use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscoveryNode;
use replisdk::core::models::cluster::ClusterSpec;
use replisdk::core::models::namespace::Namespace;

use repliagent_client::Client;
use repliagent_client_http::ClientOptions;
use repliagent_client_http::HttpClient;
use replicore_context::Context;

use crate::Factory;

/// HTTP(S) clients factory.
pub struct HttpClientFactory;

#[async_trait::async_trait]
impl Factory for HttpClientFactory {
    async fn init(
        &self,
        _: &Context,
        ns: &Namespace,
        _: &ClusterSpec,
        node: &ClusterDiscoveryNode,
    ) -> Result<Client> {
        let mut options = ClientOptions::url(&node.agent_address);
        if let Some(ca_bundle) = &ns.tls.ca_bundle {
            options.ca_bundle(ca_bundle);
        }

        let client = HttpClient::with(options)?;
        Ok(Client::from(client))
    }
}
