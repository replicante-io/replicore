use replicante_externals_mongodb::admin::ValidationResult;

use crate::backend::ValidateImpl;
use crate::Result;

/// Data and schema validation operations.
pub struct Validate {
    validate: ValidateImpl,
}

impl Validate {
    pub(crate) fn new(validate: ValidateImpl) -> Validate {
        Validate { validate }
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
