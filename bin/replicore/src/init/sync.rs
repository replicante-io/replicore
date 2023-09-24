//! RepliCore dependency Synchronisation (initialise or migrate state).
use anyhow::Result;

use replicore_conf::Conf;

use super::generic::GenericInit;

/// Process builder to initialise and run a RepliCore dependences sync process.
pub struct Sync {
    generic: GenericInit,
}

impl Sync {
    /// Build a server from the loaded configuration.
    pub async fn configure(conf: Conf) -> Result<Self> {
        let generic = GenericInit::configure(conf).await?;
        let sync = Self { generic };
        Ok(sync)
    }

    /// Finalise process initialisation and run the RepliCore server.
    pub async fn run(self) -> Result<()> {
        let logger = self.generic.telemetry.logger.clone();
        let mut generic = self.generic.run_server()?;
        generic.shutdown = generic.shutdown.watch_tokio(tokio::spawn(async {
            synchronise_dependencies(logger).await
        }));
        generic.wait().await
    }
}

/// Entrypoint to dependences synchronisation.
async fn synchronise_dependencies(logger: slog::Logger) -> Result<()> {
    slog::info!(logger, "Synchronising dependences");
    // TODO: Synchronise election service.
    // TODO: Synchronise persistence store.
    // TODO: Synchronise task submission queues.
    // TODO: Synchronise event streams.
    Ok(())
}
