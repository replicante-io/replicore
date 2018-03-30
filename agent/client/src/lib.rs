#[macro_use]
extern crate error_chain;
extern crate reqwest;

extern crate replicante_agent_models;

use replicante_agent_models::AgentInfo;


mod errors;
mod http;

#[cfg(test)]
pub mod mock;

pub use self::errors::*;
pub use self::http::HttpClient;


/// Interface to interact with (remote) agents.
///
/// Users should use the `HttpClient`.
/// The `mock` module is useful for tests.
pub trait Client {
    /// Returns general agent and datastore information.
    fn info(&self) -> Result<AgentInfo>;
}
