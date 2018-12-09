use std::sync::Arc;

use slog::Logger;

use super::BackendConfig;
use super::Config;
use super::Result;
use super::backend;
use super::backend::BackendAdmin;


/// Interface to admin distributed coordination services.
#[derive(Clone)]
pub struct Admin(Arc<BackendAdmin>);

impl Admin {
    pub fn new(config: Config, logger: Logger) -> Result<Admin> {
        let backend = match config.backend {
            BackendConfig::Zookeeper(zookeeper) => Arc::new(
                backend::zookeeper::ZookeeperAdmin::new(zookeeper, logger)?
            ),
        };
        Ok(Admin(backend))
    }

    /// Internal method to create an `Admin` from the given backend.
    pub(crate) fn with_backend(backend: Arc<BackendAdmin>) -> Admin {
        Admin(backend)
    }
}
