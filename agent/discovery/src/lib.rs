extern crate failure;
extern crate failure_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

extern crate replicante_data_models;


mod backends;
mod config;
mod error;

pub use self::backends::Iter;
pub use self::backends::discover;
pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

// Expose internal models for validaion.
pub use self::backends::DiscoveryFileModel;
