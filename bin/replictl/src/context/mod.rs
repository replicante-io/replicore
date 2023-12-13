use clap::Args;

mod store;
mod structs;

pub use store::ContextStore;
pub use structs::Connection;
pub use structs::Context;
pub use structs::Scope;

const DEFAULT_CONTEXT: &str = "default";

/// Context-related CLI options usable by any command.
#[derive(Args, Debug)]
pub struct ContextOpt {
    /// Override the cluster ID to operate on.
    #[arg(long, global = true, env = "RCTL_CLUSTER")]
    pub cluster: Option<String>,

    /// Use the specified context for all operations.
    #[arg(long = "context", global = true, env = "RCTL_CONTEXT")]
    pub name: Option<String>,

    /// Override the namespace to operate on.
    #[arg(short, long, global = true, env = "RCTL_NAMESPACE")]
    pub namespace: Option<String>,

    /// Override the node to operate on.
    #[arg(long, global = true, env = "RCTL_NODE")]
    pub node: Option<String>,
}
