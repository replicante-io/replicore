extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
extern crate prometheus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate zookeeper;

extern crate replicante_util_rndid;


mod admin;
mod backend;
mod config;
mod coordinator;
mod error;
mod metrics;
mod node_id;
mod tombstone;

#[cfg(debug_assertions)]
pub mod mock;


pub use self::admin::Admin;
pub use self::config::Backend as BackendConfig;
pub use self::config::Config;
pub use self::coordinator::Coordinator;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::node_id::NodeId;
pub use self::tombstone::Tombstone;
