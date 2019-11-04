use failure::ResultExt;
use slog::Logger;

use replicante::Config;
use replicante_service_coordinator::Admin as CoordinatorAdmin;

use super::value_or_error;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Collect version information for the configured coordinator.
pub fn version(config: &Config, logger: &Logger) -> Result<()> {
    let version = CoordinatorAdmin::new(config.coordinator.clone(), logger.clone())
        .with_context(|_| ErrorKind::AdminInit("coordinator"))
        .and_then(|admin| {
            admin
                .version()
                .with_context(|_| ErrorKind::FetchVersion("coordinator"))
        })
        .map_err(Error::from);
    println!(
        "==> Coordinator: {}",
        value_or_error(logger, "coordinator", version)
    );
    Ok(())
}
