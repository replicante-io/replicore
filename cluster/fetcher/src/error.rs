use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

use replicante_models_core::agent::AgentStatus;
use replicante_util_failure::format_fail;

pub use replicore_util_errors::AnyWrap;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    /// Return the `AgentStatus` based on the reported error.
    ///
    /// The `Error` itself can be used to determie if the issue is with the
    /// agent or with the datastore.
    ///
    /// `None` is returned if the error is with Replicante Core itself.
    ///
    /// # Examples
    ///
    ///   * An `AgentConnect` error kind results in an `AgentStatus::AgentDown`.
    ///   * A `DatastoreDown` error kind results in an `AgentStatus::NodeDown`.
    ///   * A `StoreRead` error kind results in a `None` as it indicats an error between
    ///     Replicante Core and its store and is unrelated the the agent itself.
    pub(crate) fn agent_status(&self) -> Option<AgentStatus> {
        match self.kind() {
            ErrorKind::AgentConnect(_) |
            ErrorKind::AgentDown(_, _) |
            // TODO: Consider splitting the agent state from a sync error.
            //       These are technically AgentStatus::Up but missconfigured.
            //       Alternatively add an AgentStatus::Degraded or AgentStatus::Missconfigured
            //       to support a more realistic interpretation of these.
            ErrorKind::ClusterDisplayNameDoesNotMatch(_, _, _) |
            ErrorKind::ClusterIdDoesNotMatch(_, _, _) => {
                let message = format_fail(self);
                Some(AgentStatus::AgentDown(message))
            }
            ErrorKind::DatastoreDown(_, _) => {
                let message = format_fail(self);
                Some(AgentStatus::NodeDown(message))
            }
            _ => None,
        }
    }

    /// Return more information about this error.
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "error connecting to agent {}", _0)]
    AgentConnect(String),

    #[fail(display = "error fetching/sending {} from/to agent {}", _0, _1)]
    AgentDown(&'static str, String),

    #[fail(display = "anywrap error handling rollout boundary")]
    AnyWrapped,

    #[fail(display = "error fetching/sending {} from/to agent {}", _0, _1)]
    DatastoreDown(&'static str, String),

    #[fail(
        display = "expected cluster display name '{}' but found '{}' for node with ID '{}'",
        _0, _1, _2
    )]
    ClusterDisplayNameDoesNotMatch(String, String, String),

    #[fail(
        display = "expected cluster ID '{}' but found '{}' for node with ID '{}'",
        _0, _1, _2
    )]
    ClusterIdDoesNotMatch(String, String, String),

    #[fail(display = "error updating the cluster view with info from agents")]
    ClusterViewUpdate,

    #[fail(display = "error emitting {} event", _0)]
    EventEmit(&'static str),

    #[fail(
        display = "could not find details for action {} in cluster {}.{}",
        _2, _0, _1
    )]
    // namespace_id, cluster_id, action_id
    ExpectedActionNotFound(String, String, uuid::Uuid),

    #[fail(display = "error fetching {} from the primary store", _0)]
    PrimaryStoreRead(&'static str),

    #[fail(display = "error persisting {} to the primary store", _0)]
    PrimaryStoreWrite(&'static str),

    #[fail(display = "error fetching {} from the view store", _0)]
    ViewStoreRead(&'static str),

    #[fail(display = "error persisting {} to the view store", _0)]
    ViewStoreWrite(&'static str),
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::AgentConnect(_) => "AgentConnect",
            ErrorKind::AgentDown(_, _) => "AgentDown",
            ErrorKind::AnyWrapped => "AnyWrapped",
            ErrorKind::ClusterDisplayNameDoesNotMatch(_, _, _) => "ClusterDisplayNameDoesNotMatch",
            ErrorKind::ClusterIdDoesNotMatch(_, _, _) => "ClusterIdDoesNotMatch",
            ErrorKind::ClusterViewUpdate => "ClusterViewUpdate",
            ErrorKind::DatastoreDown(_, _) => "DatastoreDown",
            ErrorKind::EventEmit(_) => "EventEmit",
            ErrorKind::ExpectedActionNotFound(_, _, _) => "ExpectedActionNotFound",
            ErrorKind::PrimaryStoreRead(_) => "PrimaryStoreRead",
            ErrorKind::PrimaryStoreWrite(_) => "PrimaryStoreWrite",
            ErrorKind::ViewStoreRead(_) => "ViewStoreRead",
            ErrorKind::ViewStoreWrite(_) => "ViewStoreWrite",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Errors encountered while computing a scheduling choice.
#[derive(thiserror::Error, Debug)]
pub enum SchedChoiceError {
    /// An orchestrator action in the cluster is not available in the registry.
    ///
    /// Attached to this error are:
    ///
    ///   * The missing orchestrator action kind.
    ///   * The cluster namespace.
    ///   * The cluster id.
    #[error("unknown orchestrator action {0} in cluster {1}.{2}")]
    OrchestratorActionNotFound(String, String, String),
}

impl SchedChoiceError {
    /// Return a `SchedChoiceError::OrchestratorActionNotFound` error.
    pub fn orchestrator_action_not_found<KIND, NS, CID>(
        kind: KIND,
        namespace: NS,
        cluster: CID,
    ) -> Self
    where
        KIND: Into<String>,
        NS: Into<String>,
        CID: Into<String>,
    {
        let kind = kind.into();
        let namespace = namespace.into();
        let cluster = cluster.into();
        SchedChoiceError::OrchestratorActionNotFound(kind, namespace, cluster)
    }
}
