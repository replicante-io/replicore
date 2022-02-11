use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::ContextOpt;

/// Information needed to access the Replicante API.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Connection {
    /// Bundle of CA certificated to validate the API server with.
    #[serde(default)]
    pub ca_bundle: Option<String>,

    /// Client key and certificate PEM bundle for mutual TLS.
    #[serde(default)]
    pub client_key: Option<String>,

    /// URL to connect to the Replicante Core API servers.
    pub url: String,
}

/// Contextual information used by API requests.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Context {
    /// How to reach the Replicante API server(s).
    pub connection: Connection,

    /// Selected scope for operations.
    #[serde(default)]
    pub scope: Scope,
}

impl Context {
    /// Get the selected cluster or fail.
    pub fn cluster(&self, opt: &ContextOpt) -> Result<String> {
        opt.cluster
            .clone()
            .or_else(|| self.scope.cluster.clone())
            .ok_or_else(|| ScopeError::NoCluster.into())
    }

    /// Get the selected namespace or fail.
    pub fn namespace(&self, opt: &ContextOpt) -> Result<String> {
        opt.namespace
            .clone()
            .or_else(|| self.scope.namespace.clone())
            .ok_or_else(|| ScopeError::NoNamespace.into())
    }

    /// Get the selected node or fail.
    pub fn node(&self, opt: &ContextOpt) -> Result<String> {
        opt.node
            .clone()
            .or_else(|| self.scope.node.clone())
            .ok_or_else(|| ScopeError::NoNode.into())
    }
}

/// Pre-selected scope for operations to target the correct namespace, cluster, ...
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Scope {
    /// The cluster to operate on, if none was explicitly set.
    #[serde(default)]
    pub cluster: Option<String>,

    /// The namespace to operate on, if none was explicitly set.
    #[serde(default)]
    pub namespace: Option<String>,

    /// The node to operate on, if none was explicitly set.
    #[serde(default)]
    pub node: Option<String>,
}

/// Errors attempting to access scopes.
#[derive(thiserror::Error, Debug)]
pub enum ScopeError {
    #[error("A cluster must be selected.\nTry adding --cluster or replictl context change")]
    NoCluster,

    #[error("A namespace must be selected.\nTry adding --namespace or replictl context change")]
    NoNamespace,

    #[error("A node must be selected.\nTry adding --node or replictl context change")]
    NoNode,
}
