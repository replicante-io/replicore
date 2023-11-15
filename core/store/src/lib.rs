//! Persistent storage interface for RepliCore Control Plane.
use std::sync::Arc;

/// TODO
#[derive(Clone)]
pub struct Store(Arc<dyn StoreBackend>);

impl<T> From<T> for Store
where
    T: StoreBackend + 'static,
{
    fn from(value: T) -> Self {
        Store(Arc::new(value))
    }
}

/// TODO
#[async_trait::async_trait]
pub trait StoreBackend: Send + Sync {
    // TODO
}

/// TODO
#[async_trait::async_trait]
pub trait StoreFactory: Send + Sync {
    // TODO
}
