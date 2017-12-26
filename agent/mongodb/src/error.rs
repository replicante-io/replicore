use mongodb;

use unamed_agent::AgentError;


pub fn to_agent(error: mongodb::error::Error) -> AgentError {
    match error {
        _ => AgentError::DatastoreError(error.to_string())
    }
}
