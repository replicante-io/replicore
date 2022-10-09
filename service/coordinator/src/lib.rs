pub mod admin;

mod backend;
mod config;
mod coordinator;
mod error;
mod metrics;
mod node_id;

#[cfg(debug_assertions)]
pub mod mock;

pub use self::admin::Admin;
pub use self::config::Backend as BackendConfig;
pub use self::config::Config;
pub use self::coordinator::Coordinator;
pub use self::coordinator::Election;
pub use self::coordinator::ElectionStatus;
pub use self::coordinator::ElectionWatch;
pub use self::coordinator::LoopingElection;
pub use self::coordinator::LoopingElectionControl;
pub use self::coordinator::LoopingElectionLogic;
pub use self::coordinator::LoopingElectionOpts;
pub use self::coordinator::NonBlockingLock;
pub use self::coordinator::NonBlockingLockWatcher;
pub use self::coordinator::ShutdownReceiver;
pub use self::coordinator::ShutdownSender;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::node_id::NodeId;
