use failure::ResultExt;
use slog::Logger;

use replicante::Config;
use replicante_service_tasks::Admin as TasksAdmin;

use replicore_models_tasks::ReplicanteQueues;

use super::value_or_error;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Collect version information for the configured tasks queue.
pub fn version(config: &Config, logger: &Logger) -> Result<()> {
    let version = TasksAdmin::<ReplicanteQueues>::new(logger.clone(), config.tasks.clone())
        .with_context(|_| ErrorKind::AdminInit("tasks queue"))
        .and_then(|tasks| {
            tasks
                .version()
                .with_context(|_| ErrorKind::FetchVersion("tasks queue"))
        })
        .map_err(Error::from);
    println!(
        "==> Tasks Queue: {}",
        value_or_error(logger, "tasks queue", version)
    );
    Ok(())
}
