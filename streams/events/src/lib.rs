#[macro_use]
extern crate error_chain;
extern crate prometheus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

extern crate replicante_data_models;
extern crate replicante_data_store;


use std::sync::Arc;

use prometheus::Registry;
use slog::Logger;

use replicante_data_models::Event;
use replicante_data_store::Store;


mod backend;
mod config;
mod errors;
mod interface;

// Cargo builds dependencies in debug mode instead of test mode.
// That means that `cfg(test)` cannot be used if the mock is used outside the crate.
#[cfg(debug_assertions)]
pub mod mock;


pub use self::config::Config;
pub use self::errors::*;

use self::interface::StreamInterface;


/// Public interface to the events streaming system.
///
/// This interface abstracts every interaction with the event streaming layer and
/// hides implementation details about straming software and data encoding.
///
/// # Backends
/// The event streaming backend is configurable to allow users to pick their preferred
/// streaming software and balance complexty, scalability, and flexibility to user needs.
#[derive(Clone)]
pub struct EventsStream(Arc<StreamInterface>);

impl EventsStream {
    pub fn new(config: Config, logger: Logger, store: Store) -> EventsStream {
        let stream = self::backend::new(config, logger, store);
        EventsStream(stream)
    }

    /// Attemps to register metrics with the Repositoy.
    ///
    /// Metrics that fail to register are logged and ignored.
    pub fn register_metrics(_logger: &Logger, _registry: &Registry) {
        // NOOP
    }
}

impl EventsStream {
    /// Emit events to the events stream.
    pub fn emit(&self, event: Event) -> Result<()> {
        self.0.emit(event)
    }
}
