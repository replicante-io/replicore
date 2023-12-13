//! CLI process errors occurring during command execution.

/// Error indicating a context does not exist.
#[derive(thiserror::Error, Debug)]
#[error("The context named '{context}' was not found")]
pub struct ContextNotFound {
    context: String,
}

impl ContextNotFound {
    /// Create a context not found error for the given name.
    pub fn for_name<S>(name: S) -> ContextNotFound
    where
        S: Into<String>,
    {
        let context = name.into();
        ContextNotFound { context }
    }

    /// The name of the context we failed to find.
    pub fn name(&self) -> &str {
        &self.context
    }
}

/// Errors attempting to access scopes.
#[derive(thiserror::Error, Debug)]
pub enum InvalidScope {
    #[error(
        "A cluster must be selected.
Try adding --cluster or set one with 'replictl context change'"
    )]
    ClusterNotSelected,

    #[error(
        "A namespace must be selected.
Try adding --namespace or set one with 'replictl context change'"
    )]
    NamespaceNotSelected,

    #[error(
        "A node must be selected.
Try adding --node or set one with 'replictl context change'"
    )]
    NodeNotSelected,
}
