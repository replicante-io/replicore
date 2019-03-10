extern crate failure;
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
extern crate prometheus;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

extern crate replicante_agent_models;


use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;


mod error;
mod http;
mod metrics;

#[cfg(test)]
pub mod mock;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::http::HttpClient;
pub use self::metrics::register_metrics;


/// Interface to interact with (remote) agents.
///
/// Users should use the `HttpClient`.
/// The `mock` module is useful for tests.
pub trait Client {
    /// Returns general agent information.
    fn agent_info(&self) -> Result<AgentInfo>;

    /// Returns general datastore information.
    fn datastore_info(&self) -> Result<DatastoreInfo>;

    /// Returns an ID that can be used to identify the agent.
    ///
    /// Mainly intended for context in error messages and introspection.
    fn id(&self) -> &str;

    /// Returns status information for the node.
    fn shards(&self) -> Result<Shards>;
}
