//! Utilities for HTTP clients from the [`reqwest`] crate.
mod config;
mod error;

pub use self::config::ClientOptions;
pub use self::config::ClientOptionsBuilder;
pub use self::error::inspect;
pub use self::error::ClientError;
pub use self::error::EmptyResponse;
pub use self::error::InvalidResponse;
pub use self::error::ResourceIdentifier;
pub use self::error::ResourceNotFound;
pub use self::error::ServerError;
