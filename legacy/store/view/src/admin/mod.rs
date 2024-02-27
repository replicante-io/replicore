use slog::Logger;

use replicante_models_core::admin::Version;

use crate::backend::backend_factory_admin;
use crate::backend::AdminImpl;
use crate::Config;
use crate::Result;

mod data;
mod validate;

use self::data::Data;
use self::validate::Validate;

/// Interface to manage Replicante view store layer.
///
/// This interface abstracts every interaction with the persistence layer and
/// hides implementation details about storage software and data encoding.
#[derive(Clone)]
pub struct Admin {
    admin: AdminImpl,
}

impl Admin {
    /// Instantiate a new storage admin interface.
    pub fn make(config: Config, logger: Logger) -> Result<Admin> {
        let admin = backend_factory_admin(config, logger)?;
        Ok(Admin { admin })
    }

    /// Data validation operations.
    pub fn data(&self) -> Data {
        let data = self.admin.data();
        Data::new(data)
    }

    /// Schema validation operations.
    pub fn validate(&self) -> Validate {
        let validate = self.admin.validate();
        Validate::new(validate)
    }

    /// Detect the version of the store in use.
    pub fn version(&self) -> Result<Version> {
        self.admin.version()
    }
}
