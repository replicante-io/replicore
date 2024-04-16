//! Orchestrate execution of orchestrator actions.
use anyhow::Result;

use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;

use crate::init::InitData;

/// Progress all already running orchestrator actions.
pub async fn progress(
    _context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
) -> Result<()> {
    for action in &data.cluster_current.oactions_unfinished {
        let action = (**action).clone();

        // Invoke the action logic to make progress.
        if action.state.is_running() {
            // TODO: lookup the action logic from kind.
            // TODO: fail if unknown kind.
            // TODO: invoke handler.
            // TODO: update record based on outcome.
            // TODO: persist action and emit events as needed.
        }

        // Carry over view of unfinished (updated) actions.
        if !action.state.is_final() {
            cluster_new.oaction(action)?;
        }
    }
    Ok(())
}
