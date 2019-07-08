use crate::backend::EventsImpl;
use crate::backend::PersistImpl;
use crate::backend::StoreImpl;
use crate::backend::StoreInterface;
use crate::store::Store;

/// Mock implementation of the `StoreInterface`.
pub struct StoreMock {
    // TODO: implement when needed.
}

impl StoreInterface for StoreMock {
    fn events(&self) -> EventsImpl {
        panic!("TODO: StoreMock::events")
    }

    fn persist(&self) -> PersistImpl {
        panic!("TODO: StoreMock::persist")
    }
}

impl From<StoreMock> for Store {
    fn from(store: StoreMock) -> Store {
        let store = StoreImpl::new(store);
        Store::with_impl(store)
    }
}
