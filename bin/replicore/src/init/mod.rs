//! Initialisation logic for Replicante Core processes.
mod generic;
mod server;
mod sync;

pub use self::generic::GenericInit;
pub use self::server::Server;
pub use self::sync::Sync;

/// ID of the replicore release in sentry recommanded format.
const RELEASE_ID: &str = concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"));
