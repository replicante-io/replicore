//! RepliCore Control Plane Server initialisation as a builder.
use anyhow::Result;

use replicore_conf::Conf;

use super::generic::GenericInit;

/// Process builder to initialise and run a RepliCore Control Plane instance.
pub struct Server {
    generic: GenericInit,
}

impl Server {
    /// Build a server from the loaded configuration.
    pub async fn configure(conf: Conf) -> Result<Self> {
        let generic = GenericInit::configure(conf).await?;
        let server = Self { generic };
        Ok(server)
    }

    /// Finalise process initialisation and run the RepliCore server.
    pub async fn run(self) -> Result<()> {
        let generic = self.generic.run_server()?;
        generic.wait().await
    }
}
