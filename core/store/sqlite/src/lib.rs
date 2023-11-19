//! Persistent store backed by an SQLite database.
//!
//! This backend is intended for small scale, single process, deployments.
//! It trades performance for convenience: easy to use and setup but can be inefficient.
//!
//! ## Shared DB file
//!
//! This backend is intended to share the same SQLite database file with other SQLite backends:
//!
//! - Table names are prefixed with `stores_` to avoid clashes.
//! - Schema migration data is kept in the `refinery_schema_history__store` table.
mod conf;
mod factory;
mod schema;
mod statements;
mod telemetry;

pub use self::conf::Conf;
pub use self::conf::ConfError;
pub use self::factory::SQLiteFactory;
