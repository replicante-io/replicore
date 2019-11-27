use serde_json::Value;

use replicante_models_core::api::apply::ApplyObject;

use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Validate the apply object for type and required attributes.
pub fn required_attributes(object: Value) -> Result<ApplyObject> {
    ApplyObject::from_raw(object)
        .map_err(ErrorKind::ValidateFailed)
        .map_err(Error::from)
}
