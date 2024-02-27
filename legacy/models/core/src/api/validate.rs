use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

/// Description of a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Error {
    pub attribute: String,
    pub code: String,
    pub message: String,
}

/// Collection of validation errors identified while checking data.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ErrorsCollection(Vec<Error>);

impl ErrorsCollection {
    /// Initialise an empty collection of errors.
    pub fn new() -> ErrorsCollection {
        ErrorsCollection(Vec::new())
    }

    /// Add a new validation error to the collection.
    pub fn collect<A, C, M>(&mut self, code: C, attribute: A, message: M)
    where
        A: Into<String>,
        C: Into<String>,
        M: Into<String>,
    {
        let attribute = attribute.into();
        let code = code.into();
        let message = message.into();
        let error = Error {
            attribute,
            code,
            message,
        };
        self.0.push(error);
    }

    /// Convert the collection of errors into a `Result`.
    ///
    /// This will return `Ok(())` if no errors where collected
    /// or call the provided `error` factory to return an `Err`.
    pub fn into_result<E, F>(self, error: F) -> Result<(), E>
    where
        F: Fn(ErrorsCollection) -> E,
    {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(error(self))
        }
    }
}

impl Deref for ErrorsCollection {
    type Target = [Error];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
