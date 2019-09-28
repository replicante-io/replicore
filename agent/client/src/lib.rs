use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_agent::actions::api::ActionInfoResponse;
use replicante_models_agent::actions::ActionListItem;
use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;

mod error;
mod http;
mod metrics;

#[cfg(any(test, feature = "with_test_support"))]
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
    /// Return all available information about an action.
    fn action_info(&self, id: &Uuid, span: Option<SpanContext>) -> Result<ActionInfoResponse>;

    /// Return a list of finished actions the agent is done processing.
    fn actions_finished(&self, span: Option<SpanContext>) -> Result<Vec<ActionListItem>>;

    /// Return a list of running or pending actions the agent has to process.
    fn actions_queue(&self, span: Option<SpanContext>) -> Result<Vec<ActionListItem>>;

    /// Return general agent information.
    fn agent_info(&self, span: Option<SpanContext>) -> Result<AgentInfo>;

    /// Return general datastore information.
    fn datastore_info(&self, span: Option<SpanContext>) -> Result<DatastoreInfo>;

    /// Return an ID that can be used to identify the agent.
    ///
    /// Mainly intended for context in error messages and introspection.
    fn id(&self) -> &str;

    /// Return status information for the node.
    fn shards(&self, span: Option<SpanContext>) -> Result<Shards>;
}
