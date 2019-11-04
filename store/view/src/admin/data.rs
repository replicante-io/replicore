use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::events::Event;

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

    /// Iterate over all actions in the store.
    pub fn actions(&self) -> Result<Cursor<Action>> {
        self.data.actions()
    }

    /// Iterate over all actions history records in the store.
    pub fn actions_history(&self) -> Result<Cursor<ActionHistory>> {
        self.data.actions_history()
    }

    /// Iterate over all events in the store.
    pub fn events(&self) -> Result<Cursor<Event>> {
        self.data.events()
    }
}
