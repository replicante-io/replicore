//! Agent client factory interface.
//!
//! Using URLs to address client provides users with a familiar configuration
//! while giving us the flexibility to support many different implementations.
use std::sync::Arc;

use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscoveryNode;
use replisdk::core::models::cluster::ClusterSpec;

use repliagent_client::Client;
use replicore_context::Context;

/// Convenience type for heap allocated [`Factory`]s.
pub type ArcedFactory = Arc<dyn Factory>;

/// Async function to initialise Agent clients on demand.
#[async_trait::async_trait]
pub trait Factory: Send + Sync {
    /// Initialise a new [`Agent`](Client) client.
    async fn init(
        &self,
        context: &Context,
        cluster: &ClusterSpec,
        node: &ClusterDiscoveryNode,
    ) -> Result<Client>;
}
