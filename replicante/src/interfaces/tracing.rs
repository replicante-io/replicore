use std::sync::Arc;

use failure::ResultExt;
use opentracingrust::Tracer;
use slog::Logger;

use replicante_util_tracing::tracer;
use replicante_util_tracing::Config;
use replicante_util_upkeep::Upkeep;

use super::ErrorKind;
use super::Result;

/// Distributed tracing interface.
#[derive(Clone)]
pub struct Tracing {
    tracer: Arc<Tracer>,
}

impl Tracing {
    /// Creates a new `Tracing` interface.
    ///
    /// Configuring the tracer usually also start the
    /// reporting thread but this is backend dependent.
    pub fn new(config: Config, logger: Logger, upkeep: &mut Upkeep) -> Result<Tracing> {
        let opts = replicante_util_tracing::Opts::new("replicore", logger, upkeep);
        let tracer = tracer(config, opts).with_context(|_| ErrorKind::InterfaceInit("tracing"))?;
        let tracer = Arc::new(tracer);
        Ok(Tracing { tracer })
    }

    /// Noop method for standard interface.
    pub fn run(&self) -> Result<()> {
        Ok(())
    }

    /// Access the stored [`Tracer`]
    ///
    /// [`Tracer`]: opentracingrust/struct.Tracer.html
    #[allow(unused)]
    pub fn tracer(&self) -> &Tracer {
        &self.tracer
    }

    /// Returns a `Tracing` instance usable as a mock.
    #[cfg(test)]
    pub fn mock() -> Tracing {
        let (tracer, _) = ::opentracingrust::tracers::NoopTracer::new();
        let tracer = Arc::new(tracer);
        Tracing { tracer }
    }
}
