use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use uuid::Uuid;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::events::Event;

use crate::backend::ActionsImpl;
use crate::backend::ActionsInterface;
use crate::backend::EventsImpl;
use crate::backend::PersistImpl;
use crate::backend::PersistInterface;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::actions::SearchFilters as ActionsSearchFilters;
use crate::store::Store;
use crate::Cursor;
use crate::Result;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    // TODO: implement when needed.
}

impl StoreInterface for StoreMock {
    fn actions(&self, _: String) -> ActionsImpl {
        let actions = Actions {};
        ActionsImpl::new(actions)
    }

    fn events(&self) -> EventsImpl {
        panic!("TODO: StoreMock::events")
    }

    fn persist(&self) -> PersistImpl {
        let persist = Persist {};
        PersistImpl::new(persist)
    }
}

impl From<StoreMock> for Store {
    fn from(store: StoreMock) -> Store {
        let store = StoreImpl::new(store);
        Store::with_impl(store)
    }
}

struct Actions {
    // TODO: implement when needed.
}

impl ActionsInterface for Actions {
    fn action(&self, _: Uuid, _: Option<SpanContext>) -> Result<Option<Action>> {
        Ok(None)
    }

    fn finish_history(&self, _: Uuid, _: DateTime<Utc>, _: Option<SpanContext>) -> Result<()> {
        Ok(())
    }

    fn history(&self, _: Uuid, _: Option<SpanContext>) -> Result<Vec<ActionHistory>> {
        Ok(Vec::new())
    }

    fn search(&self, _: ActionsSearchFilters, _: Option<SpanContext>) -> Result<Cursor<Action>> {
        panic!("TODO: MockStore::actions::search")
    }
}

struct Persist {
    // TODO: implement when needed.
}

impl PersistInterface for Persist {
    fn action(&self, _: Action, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn action_history(&self, _: Vec<ActionHistory>, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }

    fn event(&self, _: Event, _: Option<SpanContext>) -> Result<()> {
        // Noop for now.
        Ok(())
    }
}
