use thiserror::Error;

/// Error indicating an action with the given ID is already present in the registry builder.
#[derive(Error, Debug)]
#[error("orchestrator action already present in the registry: id '{id}'")]
pub struct ActionAlreadyRegistered {
    /// ID of the duplicate action being registered.
    pub id: String,
}

/// Common error for actions that can't parse the `record.args` attribute due to its type.
///
/// NOTE:
///  This error is specifically about the root `record.args` type and not for the type
///  of attributes found within the provided `record.args` object.
#[derive(Error, Debug)]
#[error("orchestrator action arguments is not of a valid expected type(s): {expected_types}")]
pub struct InvalidArgumentsType {
    /// Type or comma separated list of types the action implementation is expecting.
    pub expected_types: String,
}

/// Common error for actions that can't parse a specific argument from the provided `args`.
#[derive(Error, Debug)]
pub struct InvalidOrMissingArgument {
    /// Type or comma separated list of types the action implementation is expecting.
    pub expected_types: String,

    /// JSON path of the attribute that is missing or invalid.
    pub path: String,

    /// Indicates that the argument is required.
    pub required: bool,
}

impl std::fmt::Display for InvalidOrMissingArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.required {
            write!(
                f,
                "Required argument '{}' is missing or not of type(s) {}",
                self.path, self.expected_types
            )
        } else {
            write!(
                f,
                "Optional argument '{}' is not of type(s) {}",
                self.path, self.expected_types
            )
        }
    }
}
