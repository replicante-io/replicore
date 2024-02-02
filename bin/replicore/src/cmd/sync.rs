//! Initialise or migrate Replicante Core dependences.
use anyhow::Result;

use replicore_conf::Conf;

use super::Cli;
use crate::init::Sync;

/// Synchronise (initialise or migrate) stateful dependences so the server can run.
pub async fn run(_cli: Cli, conf: Conf) -> Result<()> {
    Sync::configure(conf)
        .await?
        .register_core_tasks()
        .register_default_backends()
        .run()
        .await
}
