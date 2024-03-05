//! Interface for a convergence step interface.
use anyhow::Result;

use replicore_context::Context;

use crate::init::InitData;

/// Interface to check the current known state of the cluster and schedule necessary actions.
#[async_trait::async_trait]
pub trait ConvergeStep: Send + Sync {
    /// Check a cluster and schedule convergence actions if needed.
    async fn converge(&self, context: &Context, data: &InitData) -> Result<()>;
}
