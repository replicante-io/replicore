use replicante_data_models::Agent;

use super::super::super::Cursor;
use super::super::super::Result;
use super::super::super::ValidationResult;
use super::super::super::validator::InnerValidator;


/// A mock implementation of the storage validator for tests.
pub struct MockValidator {
}

impl InnerValidator for MockValidator {
    fn agents(&self) -> Result<Cursor<Agent>> {
        Err("This feature is not yet mocked".into())
    }

    fn agents_count(&self) -> Result<u64> {
        Err("This feature is not yet mocked".into())
    }

    fn indexes(&self) -> Result<Vec<ValidationResult>> {
        Err("This feature is not yet mocked".into())
    }

    fn removed(&self) -> Result<Vec<ValidationResult>> {
        Err("This feature is not yet mocked".into())
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        Err("This feature is not yet mocked".into())
    }
}
