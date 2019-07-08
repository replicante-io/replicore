use super::store::Store;

mod store;

/// Manage a mocked store and admin interface.
#[derive(Clone, Default)]
pub struct Mock {
    // TODO: implement state when needed.
}

impl Mock {
    /// Return a `Store` "view" into the mock.
    pub fn store(&self) -> Store {
        let store = self::store::StoreMock {};
        store.into()
    }
}
