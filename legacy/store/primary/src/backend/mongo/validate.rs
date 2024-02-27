use failure::ResultExt;
use mongodb::sync::Client;

use replicante_externals_mongodb::admin::validate_removed_collections;
use replicante_externals_mongodb::admin::validate_schema;
use replicante_externals_mongodb::admin::ValidationResult;

use super::super::ValidateInterface;
use super::constants::REMOVED_COLLECTIONS;
use super::constants::VALIDATE_EXPECTED_COLLECTIONS;
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
    fn removed_entities(&self) -> Result<Vec<ValidationResult>> {
        let results = validate_removed_collections(&self.client, &self.db, &REMOVED_COLLECTIONS)
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(results)
    }

    fn schema(&self) -> Result<Vec<ValidationResult>> {
        let schema = validate_schema(&self.client, &self.db, &VALIDATE_EXPECTED_COLLECTIONS)
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        Ok(schema)
    }
}
