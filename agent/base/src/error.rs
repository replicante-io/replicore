use std::error::Error;
use std::fmt;

use iron::prelude::*;
use iron::status;
use iron_json_response::JsonResponse;


/// Wrapps an AgentError into a serializable struct.
///
/// This struct is filled in and used by the conversion
/// of an AgentError to an IronError.
#[derive(Serialize)]
struct AgentErrorResponse {
    error: String,
    kind: String,
}


/// Alias for the std::result::Result type that deals with `AgentError`s.
pub type AgentResult<T> = Result<T, AgentError>;


/// The error type for the `Agent` interface.
#[derive(Debug)]
pub enum AgentError {
    /// The datastore returned an error.
    DatastoreError(String),

    /// The datastore does not respect the documented model.
    ModelViolation(String),
}

impl From<AgentError> for IronError {
    fn from(error: AgentError) -> IronError {
        let mut response = Response::new();
        let code = match error {
            _ => status::InternalServerError
        };
        let payload = AgentErrorResponse {
            error: error.to_string(),
            kind: String::from(error.description())
        };
        response.set_mut(JsonResponse::json(payload)).set_mut(code);
        IronError {
            error: Box::new(error),
            response: response
        }
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AgentError::DatastoreError(ref msg) => write!(
                fmt, "Received error from datastore: {}", msg
            ),
            AgentError::ModelViolation(ref msg) => write!(
                fmt, "The datastore violated the documented model: {}", msg
            ),
        }
    }
}

impl Error for AgentError {
    fn description(&self) -> &str {
        match *self {
            AgentError::DatastoreError(_) => "DatastoreError",
            AgentError::ModelViolation(_) => "ModelViolation",
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
