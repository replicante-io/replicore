use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::api::validate::ErrorsCollection;

/// Metadata attribute setting the cluster id.
pub const SCOPE_CLUSTER: &str = "cluster";

/// Metadata attribute setting the node id.
pub const SCOPE_NODE: &str = "node";

/// Metadata attribute setting the namespace id.
pub const SCOPE_NS: &str = "namespace";

/// All attributes used to define scope.
pub const SCOPE_ATTRS: &[&str] = &[SCOPE_NS, SCOPE_CLUSTER, SCOPE_NODE];

/// Essential attributes on an `apply` object, decoded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplyObject {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: HashMap<String, Value>,
    #[serde(flatten)]
    pub attributes: HashMap<String, Value>,
}

impl ApplyObject {
    /// Attempt to decode an `ApplyObject` from a raw Object.
    ///
    /// The raw object is validated for required attributes first.
    ///
    /// Only basic validation for essential requirements is checked.
    /// Any additional `kind` or `apiVersion` related validation must
    /// be performed on the returned `ApplyObject`.
    pub fn from_raw(object: Value) -> Result<ApplyObject, ErrorsCollection> {
        let mut errors = ErrorsCollection::new();

        // Must be object.
        if !object.is_object() {
            errors.collect("TypeError", ".", "root must be an object");
            return Err(errors);
        }

        // Must have apiVersion, kind, metadata.
        match object.get("apiVersion") {
            Some(value) if value.is_string() => (),
            Some(_) => errors.collect("TypeError", "apiVersion", "apiVersion must be a string"),
            None => errors.collect("MissingAttribute", "apiVersion", "apiVersion is missing"),
        }
        match object.get("kind") {
            Some(value) if value.is_string() => (),
            Some(_) => errors.collect("TypeError", "kind", "kind must be a string"),
            None => errors.collect("MissingAttribute", "kind", "kind is missing"),
        }
        match object.get("metadata") {
            Some(value) if value.is_object() => (),
            Some(_) => errors.collect("TypeError", "metadata", "metadata must be an object"),
            None => errors.collect("MissingAttribute", "metadata", "metadata is missing"),
        }

        // Basic metadata validation:
        if let Some(metadata) = object.get("metadata") {
            // Scope attributes must be strings, if defined.
            for scope in SCOPE_ATTRS {
                match metadata.get(scope) {
                    None => (),
                    Some(value) if value.is_string() => (),
                    Some(_) => errors.collect(
                        "TypeError",
                        format!("metadata.{}", scope),
                        format!("metadata.{} must be a string", scope),
                    ),
                }
            }
        }

        // Decode as an apply object if validation passes.
        errors.into_result(|error| error)?;
        let object: ApplyObject = serde_json::from_value(object)
            .expect("ApplyObject decoding must be successful if validation passed");
        Ok(object)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ApplyObject;

    #[test]
    fn expect_object() {
        let object = json!("not an object");
        let errors = match ApplyObject::from_raw(object) {
            Ok(object) => panic!("unexpected apply object: {:?}", object),
            Err(errors) => errors,
        };
        assert_eq!(1, errors.len());
        assert_eq!(".", errors[0].attribute);
        assert_eq!("TypeError", errors[0].code);
        assert!(
            errors.into_result(|e| e).is_err(),
            "validation should have failed"
        );
    }

    #[test]
    fn expect_required_fields() {
        let object = json!({ "apiVersion": 1234 });
        let errors = match ApplyObject::from_raw(object) {
            Ok(object) => panic!("unexpected apply object: {:?}", object),
            Err(errors) => errors,
        };
        assert_eq!(3, errors.len());
        assert_eq!("apiVersion", errors[0].attribute);
        assert_eq!("TypeError", errors[0].code);
        assert_eq!("kind", errors[1].attribute);
        assert_eq!("MissingAttribute", errors[1].code);
        assert_eq!("metadata", errors[2].attribute);
        assert_eq!("MissingAttribute", errors[2].code);
        assert!(
            errors.into_result(|e| e).is_err(),
            "validation should have failed"
        );
    }

    #[test]
    fn pass() {
        let object = json!({
            "apiVersion": "some.api/v1",
            "kind": "TestObject",
            "metadata": {},
        });
        let object = ApplyObject::from_raw(object);
        assert!(object.is_ok(), "validation should have passed");
    }
}
