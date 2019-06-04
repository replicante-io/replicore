extern crate failure;
extern crate failure_derive;
extern crate lazy_static;
extern crate prometheus;
extern crate serde;
extern crate serde_derive;
extern crate serde_yaml;
extern crate slog;

extern crate replicante_data_models;

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
