//! Registry and factories to initialise Agent API clients on demand.
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscoveryNode;
use replisdk::core::models::cluster::ClusterSpec;

use repliagent_client::Client;
use replicore_context::Context;

mod factory;

pub mod errors;
pub mod http;

pub use self::factory::Factory;

use self::factory::ArcedFactory;

/// Registry of Agent API client factories.
#[derive(Clone)]
pub struct AgentClients {
    schemas: HashMap<String, ArcedFactory>,
}

impl AgentClients {
    /// Create an [`AgentClients`] registry with no factories configured.
    pub fn empty() -> AgentClients {
        AgentClients {
            schemas: Default::default(),
        }
    }

    /// Initialise a client to interact with an [`Agent`].
    pub async fn factory(
        &self,
        context: &Context,
        cluster: &ClusterSpec,
        node: &ClusterDiscoveryNode,
    ) -> Result<Client> {
        let (schema, _) = match node.agent_address.split_once(':') {
            Some(parts) => parts,
            None => {
                let error = self::errors::ClientNoSchema {
                    ns_id: cluster.ns_id.clone(),
                    cluster_id: cluster.cluster_id.clone(),
                    node_id: node.node_id.clone(),
                };
                anyhow::bail!(error);
            }
        };
        let factory = match self.schemas.get(schema) {
            Some(factory) => factory,
            None => {
                let error = self::errors::ClientUnknownSchema {
                    ns_id: cluster.ns_id.clone(),
                    cluster_id: cluster.cluster_id.clone(),
                    node_id: node.node_id.clone(),
                    schema: schema.to_string(),
                };
                anyhow::bail!(error);
            }
        };
        factory.init(context, cluster, node).await
    }

    /// Register a client factory for a schema.
    pub fn with_factory<F, S>(&mut self, schema: S, factory: F) -> &mut Self
    where
        S: Into<String>,
        F: Factory + 'static,
    {
        let schema = schema.into();
        let factory = Arc::new(factory);
        self.schemas.insert(schema, factory);
        self
    }
}

impl Default for AgentClients {
    fn default() -> Self {
        AgentClients::empty()
    }
}
