use anyhow::Result;

use replicante_models_core::actions::orchestrator::OrchestratorAction as OARecord;
use replicante_models_core::actions::orchestrator::OrchestratorActionScheduleMode;
use replicore_iface_orchestrator_action::OrchestratorAction;
use replicore_iface_orchestrator_action::OrchestratorActionRegistryBuilder;
use replicore_iface_orchestrator_action::ProgressChanges;

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

mod args;
mod errors;
mod request;
mod response;

#[cfg(test)]
mod tests;

/// Register debug actions with the given registry builder.
pub fn register(builder: &mut OrchestratorActionRegistryBuilder) -> Result<()> {
    builder.register("core.replicante.io/http", Http::registry_entry())?;
    Ok(())
}

/// Execute an externally implemented action over HTTP(S).
///
/// When progressed the action will make a POST HTTP(S) call to the provided URL.
/// The full action record is sent in the request body as JSON.
///
/// The response is expected to be:
///
///  - A 200, with a JSON encoded `ProgressChanges` object.
///  - A 204, which will correspond to a progress cycle which makes no changes.
///
/// Any other response will fail the action with a remote error.
///
/// If an action fails the remote should still send back a 200 with a `ProgressChanges`
/// object setting the state to `OrchestratorActionState::Failed` and include failure information.
///
/// ## Arguments
///
/// | Argument | Type | Description | Default |
/// | -------- | ---- | ----------- | ------- |
/// | `remote.ca` | String | Optional CA to validate HTTPS server certificates | None |
/// | `remote.timeout` | u64 | Optional timeout timeout to wait for a response, in seconds. | <Not Set> |
/// | `remote.url` | String | URL of the remote system to invoke | <Required> |
#[derive(Default)]
struct Http {}

replicore_iface_orchestrator_action::registry_entry_factory! {
    handler: Http,
    schedule_mode: OrchestratorActionScheduleMode::Exclusive,
    summary: "Execute an externally implemented action over HTTP(S)",
    timeout: ONE_DAY,
}

impl OrchestratorAction for Http {
    fn progress(&self, record: &OARecord) -> Result<Option<ProgressChanges>> {
        let args = self::args::Args::decode(record)?;
        let response = self::request::perform(record, &args)?;
        let changes = self::response::decode(response);
        let changes = self::response::ensure_move_to_running(record, changes);
        Ok(changes)
    }
}
