use anyhow::Result;
use uuid::Uuid;

use replicante_store_primary::store::Store;
use replicore_iface_orchestrator_action::OrchestratorActionRegistry;

use super::utils::fail_action;
use crate::ClusterOrchestrate;
use crate::ClusterOrchestrateMut;

/// Progress a running orchestration action.
pub fn continue_action(
    data: &ClusterOrchestrate,
    data_mut: &mut ClusterOrchestrateMut,
    action_id: Uuid,
    store: &Store,
) -> Result<()> {
    // Get the action and the action implementation handler and metadata.
    let action_record = super::utils::get_orchestrator_action(data, data_mut, action_id)?;
    let registry = OrchestratorActionRegistry::current();
    let action = registry.lookup(&action_record.kind);
    let action = match action {
        None => {
            let error = anyhow::anyhow!(crate::errors::ActionError::unknown_kind(
                &data.namespace.ns_id,
                &data.cluster_view.cluster_id,
                action_id,
                &action_record.kind,
            ));
            return fail_action(data, data_mut, action_record, error);
        }
        Some(action) => action,
    };

    // Call shared start/progress action logic.
    super::progress::run_action(data, data_mut, action_record, action, store)?;
    Ok(())
}
