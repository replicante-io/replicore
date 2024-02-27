use std::sync::Arc;

use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use opentracingrust::Tracer;
use slog::debug;
use slog::Logger;

use replicante_store_view::store::Store;
use replicante_stream_events::Stream;
use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_upkeep::Upkeep;

mod by_event;
mod error;
mod follower;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

/// Update the view DB based on the events stream.
pub struct ViewUpdater {
    events: Stream,
    logger: Logger,
    store: Store,
    tracer: Arc<Tracer>,
}

impl ViewUpdater {
    pub fn new(events: Stream, logger: Logger, store: Store, tracer: Arc<Tracer>) -> ViewUpdater {
        ViewUpdater {
            events,
            logger,
            store,
            tracer,
        }
    }

    /// Start the component in a background thread and return.
    pub fn run(&self, upkeep: &mut Upkeep) -> Result<()> {
        let events = self.events.clone();
        let logger = self.logger.clone();
        let store = self.store.clone();
        let tracer = self.tracer.clone();
        debug!(logger, "Starting view DB updater thread");
        let thread = ThreadBuilder::new("r:c:viewupdater")
            .full_name("replicore:component:viewupdater")
            .spawn(move |scope| {
                let thread = &scope;
                let worker = self::follower::Follower {
                    events,
                    logger: logger.clone(),
                    store,
                    thread,
                    tracer,
                };
                if let Err(error) = worker.update_view_db() {
                    capture_fail!(
                        &error,
                        logger,
                        "View DB updater stopped";
                        failure_info(&error),
                    );
                }
            })
            .with_context(|_| ErrorKind::ThreadSpawn)?;
        upkeep.register_thread(thread);
        Ok(())
    }
}
