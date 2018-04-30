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

    /// Instantiate a `Validator` that wraps the given `MockValidator`.
    // Cargo builds dependencies in debug mode instead of test mode.
    // That means that `cfg(test)` cannot be used if the mock is used outside the crate.
    #[cfg(debug_assertions)]
    pub fn mock(inner: Arc<super::mock::MockValidator>) -> Validator {
        Validator(inner)
    }
}
