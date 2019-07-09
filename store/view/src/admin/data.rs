use replicante_models_core::Event;

use crate::backend::DataImpl;
use crate::Cursor;
use crate::Result;

/// Data validation operations.
pub struct Data {
    data: DataImpl,
}

impl Data {
    pub(crate) fn new(data: DataImpl) -> Data {
        Data { data }
    }

    /// Iterate over all events in the store.
    pub fn events(&self) -> Result<Cursor<Event>> {
        self.data.events()
    }
}
