/// Orchestrator actions implement this trait to describe and execute actions.
///
/// Implementations of `OrchestratorAction`s have to be `Send` and `Sync` as
/// action progression can be invoked by any number of threads concurrently.
pub trait OrchestratorAction: Send + Sync {}
