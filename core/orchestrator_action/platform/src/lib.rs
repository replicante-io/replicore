use anyhow::Result;

use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;

mod errors;
mod node;

/// Register `Platform` actions with the given registry builder.
pub fn register(builder: &mut OrchestratorActionRegistryBuilder) -> Result<()> {
    builder.register(
        "platform.replicante.io/node.provision",
        node::Provision::registry_entry(),
    )?;
    Ok(())
}
