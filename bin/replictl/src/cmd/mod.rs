//! CLI interface for the Replicante Control Plane client.
use clap::Parser;
use clap::Subcommand;

pub mod apply;
pub mod context;
pub mod namespace;

use crate::context::ContextOpt;
use crate::formatter::FormatOpts;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// CLI definition for the replictl binary.
#[derive(Debug, Parser)]
#[command(about)]
#[command(propagate_version = true)]
#[command(version = VERSION)]
pub struct Cli {
    /// RepliCore server context selection and override arguments.
    #[command(flatten)]
    pub context: ContextOpt,

    /// Select the `replictl` command to run.
    #[command(subcommand)]
    pub command: Command,

    /// Configure how `replictl` output is formatted.
    #[command(flatten)]
    pub format: FormatOpts,
}

/// Select the `replictl` command to run.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Apply objects declarations to the Control Plane.
    Apply(apply::ApplyCli),

    /// Manage configuration of RepliCore servers to access.
    Context(context::ContextCli),

    /// Inspect, delete or manipulate namespaces.
    #[command(alias = "ns")]
    Namespace(namespace::NamespaceCli),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn clap_integrity_check() {
        let command = crate::Cli::command();
        command.debug_assert();
    }
}
