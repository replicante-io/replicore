use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;

mod change;
mod describe;
mod list;
mod login;
mod logout;
mod select;

/// Commands to operate on `replictl` contexts.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Update the context's scope attributes.
    Change,

    /// Descibe the active context.
    Describe,

    /// List known contexts.
    List,

    /// Connect to Replicante API server(s) or update connection details.
    Login,

    /// Forget how to connect to a Replicante API server and remove its context.
    Logout,

    /// Select a context to be the active context.
    Select,
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, context_cmd: &Opt) -> Result<i32> {
    match &context_cmd {
        Opt::Change => change::execute(logger, opt).await,
        Opt::Describe => describe::execute(logger, opt).await,
        Opt::List => list::execute(logger, opt).await,
        Opt::Login => login::execute(logger, opt).await,
        Opt::Logout => logout::execute(logger, opt).await,
        Opt::Select => select::execute(logger, opt).await,
    }
}
