//! Exclusive Control Plane tasks for scheduling of other tasks.

mod discovery;
mod orchestrator;

pub use self::discovery::Discovery;
pub use self::orchestrator::Orchestrate;
