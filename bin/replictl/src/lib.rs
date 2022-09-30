use anyhow::Result;
use slog::debug;
use slog::error;
use structopt::StructOpt;

use replicante_models_core::api::validate::ErrorsCollection;

mod apiclient;
mod commands;
mod context;
mod logging;
mod utils;

// Re-export errors so main can provide more accurate messages.
pub use apiclient::ApiNotFound;
pub use context::ContextNotFound;
pub use context::ScopeError;

use commands::Command;
use context::ContextOpt;
use logging::LogOpt;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " [",
    env!("GIT_BUILD_HASH"),
    "; ",
    env!("GIT_BUILD_TAINT"),
    "]",
);

/// Apply attempted on an invalid object.
#[derive(thiserror::Error, Debug)]
#[error("Apply attempted on an invalid object")]
pub struct InvalidApply {
    errors: replicante_models_core::api::validate::ErrorsCollection,
}

impl InvalidApply {
    pub fn new(errors: ErrorsCollection) -> InvalidApply {
        InvalidApply { errors }
    }
}

impl std::ops::Deref for InvalidApply {
    type Target = [replicante_models_core::api::validate::Error];
    fn deref(&self) -> &Self::Target {
        self.errors.deref()
    }
}

/// Replicante Core command line client.
#[derive(Debug, StructOpt)]
#[structopt(version = VERSION)]
pub struct Opt {
    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(flatten)]
    pub context: ContextOpt,

    #[structopt(flatten)]
    pub log: LogOpt,
}

pub fn run() -> Result<i32> {
    // Parse args and initialise logging.
    let opt = Opt::from_args();
    let logger = logging::configure(&opt.log)?;
    debug!(
        logger,
        "replictl starting";
        "git-taint" => env!("GIT_BUILD_TAINT"),
        "git-commit" => env!("GIT_BUILD_HASH"),
        "version" => env!("CARGO_PKG_VERSION"),
    );

    // Set up a tokio runtime.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("replictl-tokio-worker")
        .build()
        .expect("tokio runtime initialisation failed");
    let result = runtime.block_on(commands::execute(&logger, &opt));
    // Once done, ensure the runtime shuts down in a timely manner.
    // Note: this only effects blocking tasks and not futures.
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));

    // Can finally exit.
    if result.is_err() {
        error!(logger, "replicli exiting with error");
    } else {
        debug!(logger, "replicli exiting successfully");
    }
    result
}
