use anyhow::Result;

use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;

pub mod counting;
pub mod fail;
pub mod ping;
pub mod success;

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
const ONE_HOUR: std::time::Duration = std::time::Duration::from_secs(60 * 60);

/// Register debug actions with the given registry builder.
pub fn register(builder: &mut OrchestratorActionRegistryBuilder) -> Result<()> {
    builder.register(
        "core.replicante.io/debug.counting",
        counting::Counting::registry_entry(),
    )?;
    builder.register(
        "core.replicante.io/debug.fail",
        fail::Fail::registry_entry(),
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
