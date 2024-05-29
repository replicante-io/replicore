//! Async client library to interact with the Replicante Control Plane API.
pub use repliclient_utils::ClientOptions;

mod client;

pub use self::client::Client;
