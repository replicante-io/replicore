#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate prometheus;
extern crate reqwest;
#[macro_use]
extern crate slog;

extern crate replicante_agent_models;

use replicante_agent_models::NodeInfo;
use replicante_agent_models::NodeStatus;


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
    fn info(&self) -> Result<NodeInfo>;

    /// Returns status information for the node.
    fn status(&self) -> Result<NodeStatus>;
}
