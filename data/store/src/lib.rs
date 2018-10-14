#[macro_use]
extern crate bson;
extern crate chrono;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mongodb;

extern crate prometheus;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;

#[cfg(test)]
extern crate replicante_agent_models;
extern crate replicante_data_models;


mod backend;
mod config;
mod errors;
mod store;
mod validator;

// Cargo builds dependencies in debug mode instead of test mode.
// That means that `cfg(test)` cannot be used if the mock is used outside the crate.
#[cfg(debug_assertions)]
pub use self::backend::mock;

pub use self::config::Config;
pub use self::errors::*;

pub use self::store::EventsFilters;
pub use self::store::EventsOptions;
pub use self::store::Store;

pub use self::validator::ValidationResult;
pub use self::validator::Validator;


/// Iterator over models in the store.
pub struct Cursor<Model>(Box<Iterator<Item=Result<Model>>>);

impl<Model> Iterator for Cursor<Model> {
    type Item = Result<Model>;
    fn next(&mut self) -> Option<Result<Model>> {
        self.0.next()
    }
}
