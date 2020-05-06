use bson::doc;
use failure::ResultExt;
use mongodb::Client;

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
pub fn version(client: &Client, db: &str) -> Result<Version> {
    let version = client
        .database(db)
        .run_command(doc! {"buildInfo": 1}, None)
        .with_context(|_| ErrorKind::Version)?;
    let version = version
        .get_str("version")
        .with_context(|_| ErrorKind::Version)?;
    let version = semver::Version::parse(version).with_context(|_| ErrorKind::Version)?;
    let version = Version::new("MongoDB", version);
    Ok(version)
}
