extern crate bson;
extern crate chrono;
extern crate failure;
extern crate lazy_static;
extern crate mongodb;
extern crate opentracingrust;
extern crate prometheus;
extern crate regex;
extern crate semver;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate slog;

extern crate replicante_data_models;
extern crate replicante_util_tracing;

mod backend;
mod config;
mod error;
mod metrics;

pub mod admin;
#[cfg(feature = "with_test_support")]
pub mod mock;
pub mod store;

pub use self::config::Config;
pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;

/// Iterator over models in the store.
pub struct Cursor<Model>(Box<dyn Iterator<Item = Result<Model>>>);

impl<Model> Iterator for Cursor<Model> {
    type Item = Result<Model>;
    fn next(&mut self) -> Option<Result<Model>> {
        self.0.next()
    }
}
