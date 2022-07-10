use anyhow::Result;

use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;

pub mod counting;
pub mod failing;
pub mod ping;
pub mod success;

/// Register debug actions with the given registry builder.
pub fn register(builder: &mut OrchestratorActionRegistryBuilder) -> Result<()> {
    builder.register(
        "core.replicante.io/debug.counting",
        counting::Counting::registry_entry(),
    )?;
    builder.register(
        "core.replicante.io/debug.failing",
        failing::Failing::registry_entry(),
    )?;
    builder.register(
        "core.replicante.io/debug.ping",
        ping::Ping::registry_entry(),
    )?;
    builder.register(
        "core.replicante.io/debug.success",
        success::Success::registry_entry(),
    )?;
    Ok(())
}
