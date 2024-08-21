//! Errors reported by the convergence steps.

/// The search for node to run cluster.init on returned no target.
#[derive(Debug, thiserror::Error)]
#[error("the search for node to run cluster.init on returned no target")]
pub struct ClusterInitNoTarget;
