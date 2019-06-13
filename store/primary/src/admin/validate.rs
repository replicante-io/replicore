use super::super::backend::ValidateImpl;
use super::super::Result;
use super::ValidationResult;

/// Data and schema validation operations.
pub struct Validate {
    validate: ValidateImpl,
}

impl Validate {
    pub(crate) fn new(validate: ValidateImpl) -> Validate {
        Validate { validate }
    }

    /// Validate the current indexes to ensure they matches the code.
    pub fn indexes(&self) -> Result<Vec<ValidationResult>> {
        self.validate.indexes()
    }

    /// Checks the store for collections/tables or indexes that are no longer used.
    pub fn removed_entities(&self) -> Result<Vec<ValidationResult>> {
        self.validate.removed_entities()
    }

    /// Validate the current schema to ensure it matches the code.
    pub fn schema(&self) -> Result<Vec<ValidationResult>> {
        self.validate.schema()
    }
}
