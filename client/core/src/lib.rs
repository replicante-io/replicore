//! Async client library to interact with the Replicante Control Plane API.

mod client;
mod config;

pub mod error;

pub use self::client::Client;
pub use self::config::ClientOptions;
pub use self::config::ClientOptionsBuilder;
