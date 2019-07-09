use failure::ResultExt;
use mongodb::Client;

use replicante_externals_mongodb::admin::validate_indexes;
use replicante_externals_mongodb::admin::validate_schema;
use replicante_externals_mongodb::admin::ValidationResult;

use super::super::ValidateInterface;
use super::constants::VALIDATE_EXPECTED_COLLECTIONS;
use super::constants::VALIDATE_INDEXES_NEEDED;
use super::constants::VALIDATE_INDEXES_SUGGESTED;
use crate::ErrorKind;
use crate::Result;

/// Validation operations implementation using MongoDB.
pub struct Validate {
    client: Client,
    db: String,
}

impl Validate {
    pub fn new(client: Client, db: String) -> Validate {
        Validate { client, db }
    }
}

impl ValidateInterface for Validate {
    fn indexes(&self) -> Result<Vec<ValidationResult>> {
        let result = validate_indexes(
            &self.client,
            &self.db,
            &VALIDATE_EXPECTED_COLLECTIONS,
            &VALIDATE_INDEXES_NEEDED,
            &VALIDATE_INDEXES_SUGGESTED,
        )
        .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(result)
    }

    fn removed_entities(&self) -> Result<Vec<ValidationResult>> {
        // There is nothing removed yet.
        Ok(vec![])
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        let schema = validate_schema(&self.client, &self.db, &VALIDATE_EXPECTED_COLLECTIONS)
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(schema)
    }
}
