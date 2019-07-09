use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use slog::warn;
use slog::Logger;

use replicante_models_core::admin::Version;

mod config;
mod error;
mod healthcheck;

pub mod admin;
pub mod metrics;
pub mod operations;

pub use self::config::CommonConfig;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::healthcheck::MongoDBHealthCheck;
pub use self::metrics::register_metrics;

/// Detect the version of the MongoDB store in use.
pub fn version(client: &Client, db: &str, logger: &Logger) -> Result<Version> {
    let db = client.db(db);
    let version = db.version().with_context(|_| ErrorKind::Version)?;
    // The mongodb crate uses semver ^0.8.0 while replicante uses latest.
    // "Convert" the version object across crate versions.
    let version: semver::Version = match version.to_string().parse() {
        Ok(version) => version,
        Err(_) => {
            warn!(logger, "Failed to convert response to semver"; "version" => %version);
            semver::Version::new(version.major, version.minor, version.patch)
        }
    };
    let version = Version::new("MongoDB", version);
    Ok(version)
}
