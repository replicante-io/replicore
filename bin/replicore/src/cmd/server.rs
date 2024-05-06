//! Run Replicante Core server.
use anyhow::Result;

use replicore_conf::Conf;

use super::Cli;
use crate::init::Server;

/// Run the Replicante Core control plane server.
pub async fn run(_cli: Cli, conf: Conf) -> Result<()> {
    Server::configure(conf)
        .await?
        .register_core_tasks()
        .register_default_backends()
        .register_default_oactions()
        .with_http_config(crate::api::configure)
        .run()
        .await
}
