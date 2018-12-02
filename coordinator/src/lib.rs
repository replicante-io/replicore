extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate slog;

extern crate replicante_util_rndid;


use slog::Logger;


mod config;
mod error;
mod node_id;

#[cfg(debug_assertions)]
pub mod mock;


pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::node_id::NodeId;


/// Interface to access distributed coordination services.
#[derive(Clone)]
pub struct Coordinator();

impl Coordinator {
    pub fn new(_config: Config, _logger: Logger) -> Coordinator {
        Coordinator()
    }
}
