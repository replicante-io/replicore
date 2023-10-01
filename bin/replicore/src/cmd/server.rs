//! Run Replicante Core server.
use anyhow::Result;

use replicore_conf::Conf;

use super::Cli;
use crate::init::Server;

/// Run the Replicante Core control plane server.
pub async fn run(_cli: Cli, conf: Conf) -> Result<()> {
    Server::configure(conf)
        .await?
        .add_default_backends()
        .run()
        .await
}
