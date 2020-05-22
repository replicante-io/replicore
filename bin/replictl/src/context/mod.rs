use structopt::StructOpt;

mod store;
mod structs;

pub use store::ContextStore;
pub use structs::Connection;
pub use structs::Context;
pub use structs::Scope;
pub use structs::ScopeError;

const DEFAULT_CONTEXT: &str = "default";

/// Context-related CLI options.
#[derive(Debug, StructOpt)]
pub struct ContextOpt {
    /// Override the cluster ID to operate on.
    #[structopt(long, global = true, env = "RCTL_CLUSTER")]
    pub cluster: Option<String>,

    /// Use the specified context for all operations.
    #[structopt(long = "context", global = true, env = "RCTL_CONTEXT")]
    pub name: Option<String>,

    /// Override the namespace to operate on.
    #[structopt(short, long, global = true, env = "RCTL_NAMESPACE")]
    pub namespace: Option<String>,

    /// Override the node to operate on.
    #[structopt(long, global = true, env = "RCTL_NODE")]
    pub node: Option<String>,
}

/// Error indicating a context does not exist.
#[derive(thiserror::Error, Debug)]
#[error("A context named '{context}' was not found")]
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
