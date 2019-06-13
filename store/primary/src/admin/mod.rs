use slog::Logger;

use replicante_models_core::admin::Version;

use super::backend::backend_factory_admin;
use super::backend::AdminImpl;
use super::Config;
use super::Result;

mod data;
mod validate;

use self::data::Data;
use self::validate::Validate;

/// Interface to manage Replicante primary store layer.
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

/// Details of issues detected by the validation process.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValidationResult {
    pub collection: String,
    pub error: bool,
    pub group: &'static str,
    pub message: String,
}

impl ValidationResult {
    /// Create a `ValidationResult` for an error.
    pub fn error<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: true,
            group,
            message: message.into(),
        }
    }

    /// Create a `ValidationResult` for a non-critical issue or a suggestion.
    pub fn result<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        ValidationResult {
            collection: collection.into(),
            error: false,
            group,
            message: message.into(),
        }
    }
}
