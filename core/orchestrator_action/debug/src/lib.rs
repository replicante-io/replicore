use anyhow::Result;

use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;

pub mod counting;
pub mod failing;
pub mod ping;
pub mod success;

/// Register debug actions with the given registry builder.
pub fn register(builder: &mut OrchestratorActionRegistryBuilder) -> Result<()> {
    builder.register_type::<counting::Counting>("core.replicante.io/debug.counting")?;
    builder.register_type::<failing::Failing>("core.replicante.io/debug.failing")?;
    builder.register_type::<ping::Ping>("core.replicante.io/debug.ping")?;
    builder.register_type::<success::Success>("core.replicante.io/debug.success")?;
    Ok(())
}
