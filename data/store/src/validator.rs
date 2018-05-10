use std::sync::Arc;

use prometheus::Registry;
use slog::Logger;

use super::Config;
use super::Result;

use super::backend::mongo::MongoValidator;


/// Private interface to the persistence storage validation.
///
/// Allows multiple possible datastores to be used as well as mocks for testing.
pub trait InnerValidator: Send + Sync {
    /// See `Validator::indexes` for details.
    fn indexes(&self) -> Result<Vec<ValidationResult>>;

    /// See `Validator::removed` for details.
    fn removed(&self) -> Result<Vec<ValidationResult>>;

    /// See `Validator::schema` for details.
    fn schema(&self) -> Result<Vec<ValidationResult>>;
}


/// Details of issues detected by the validation process.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ValidationResult {
    pub collection: String,
    pub error: bool,
    pub group: &'static str,
    pub message: String,
}

impl ValidationResult {
    /// Create a `ValidationResult` for an error.
    pub fn error<S1, S2>(collection: S1, message: S2, group: &'static str) -> ValidationResult
        where S1: Into<String>,
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
        where S1: Into<String>,
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


/// Public interface to the persistent storage validation.
///
/// This interface abstracts away details about access to stored models to allow
/// for validation logic to be implemented on top of any supported datastore.
#[derive(Clone)]
pub struct Validator(Arc<InnerValidator>);

impl Validator {
    /// Instantiate a new storage validator.
    pub fn new(config: Config, logger: Logger, registry: &Registry) -> Result<Validator> {
        let validator = match config {
            Config::MongoDB(config) => Arc::new(MongoValidator::new(config, logger, registry)?),
        };
        Ok(Validator(validator))
    }

    /// Validate the current indexes to ensure they matches the code.
    pub fn indexes(&self) -> Result<Vec<ValidationResult>> {
        self.0.indexes()
    }

    /// Checks the store for collections/tables or indexes that are no longer used.
    pub fn removed(&self) -> Result<Vec<ValidationResult>> {
        self.0.removed()
    }

    /// Validate the current schema to ensure it matches the code.
    pub fn schema(&self) -> Result<Vec<ValidationResult>> {
        self.0.schema()
    }

    /// Instantiate a `Validator` that wraps the given `MockValidator`.
    // Cargo builds dependencies in debug mode instead of test mode.
    // That means that `cfg(test)` cannot be used if the mock is used outside the crate.
    #[cfg(debug_assertions)]
    pub fn mock(inner: Arc<super::mock::MockValidator>) -> Validator {
        Validator(inner)
    }
}
