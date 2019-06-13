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
