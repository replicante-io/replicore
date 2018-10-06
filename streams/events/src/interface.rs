use replicante_data_models::Event;

use super::Result;


/// Private interface to the events streaming system.
///
/// Allows multiple possible backends to be used as well as mocks for testing.
pub trait StreamInterface: Send + Sync {
    /// Emit events to the events stream.
    fn emit(&self, event: Event) -> Result<()>;
}
