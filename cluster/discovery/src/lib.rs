mod backends;
mod config;
mod error;
mod metrics;

pub use self::backends::discover;
pub use self::backends::Iter;
pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

// Expose internal models for validaion.
pub use self::backends::DiscoveryFileModel;
