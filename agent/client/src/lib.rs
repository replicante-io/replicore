extern crate failure;
extern crate failure_derive;
extern crate lazy_static;
extern crate opentracingrust;
extern crate prometheus;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate slog;

extern crate replicante_models_agent;

use opentracingrust::SpanContext;

use replicante_models_agent::AgentInfo;
use replicante_models_agent::DatastoreInfo;
use replicante_models_agent::Shards;

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
    fn agent_info(&self, span: Option<SpanContext>) -> Result<AgentInfo>;

    /// Returns general datastore information.
    fn datastore_info(&self, span: Option<SpanContext>) -> Result<DatastoreInfo>;

    /// Returns an ID that can be used to identify the agent.
    ///
    /// Mainly intended for context in error messages and introspection.
    fn id(&self) -> &str;

    /// Returns status information for the node.
    fn shards(&self, span: Option<SpanContext>) -> Result<Shards>;
}
