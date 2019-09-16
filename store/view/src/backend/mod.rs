use std::ops::Deref;
use std::sync::Arc;

use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::Logger;

use replicante_externals_mongodb::admin::ValidationResult;
use replicante_models_core::admin::Version;
use replicante_models_core::events::Event;
use replicante_service_healthcheck::HealthChecks;

use crate::store::events::EventsFilters;
use crate::store::events::EventsOptions;
use crate::Config;
use crate::Cursor;
use crate::Result;

mod mongo;

/// Instantiate a new storage backend based on the given configuration.
pub fn backend_factory<T>(
    config: Config,
    logger: Logger,
    healthchecks: &mut HealthChecks,
    tracer: T,
) -> Result<StoreImpl>
where
    T: Into<Option<Arc<Tracer>>>,
{
    let store = match config {
        Config::MongoDB(config) => {
            let store = self::mongo::Store::new(config, logger, healthchecks, tracer)?;
            StoreImpl::new(store)
        }
    };
    Ok(store)
}

/// Definition of events operations.
///
/// See `store::events::Events` for descriptions of methods.
pub trait EventsInterface: Send + Sync {
    fn range(
        &self,
        filters: EventsFilters,
        options: EventsOptions,
        span: Option<SpanContext>,
    ) -> Result<Cursor<Event>>;
}

/// Instantiate a new storage admin backend based on the given configuration.
pub fn backend_factory_admin(config: Config, logger: Logger) -> Result<AdminImpl> {
    let admin = match config {
        Config::MongoDB(config) => AdminImpl::new(self::mongo::Admin::make(config, logger)?),
    };
    Ok(admin)
}

/// Definition of top level store administration operations.
///
/// Mainly a way to return interfaces to grouped store operations.
///
/// See `admin::Admin` for descriptions of methods.
pub trait AdminInterface: Send + Sync {
    fn data(&self) -> DataImpl;
    fn validate(&self) -> ValidateImpl;
    fn version(&self) -> Result<Version>;
}

/// Dynamic dispatch all operations to a backend-specific implementation.
#[derive(Clone)]
pub struct AdminImpl(Arc<dyn AdminInterface>);

impl AdminImpl {
    pub fn new<A: AdminInterface + 'static>(admin: A) -> AdminImpl {
        AdminImpl(Arc::new(admin))
    }
}

impl Deref for AdminImpl {
    type Target = dyn AdminInterface + 'static;
    fn deref(&self) -> &(dyn AdminInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported data admin operations.
///
/// See `admin::data::Data` for descriptions of methods.
pub trait DataInterface: Send + Sync {
    fn events(&self) -> Result<Cursor<Event>>;
}

/// Dynamic dispatch all data admin operations to a backend-specific implementation.
#[derive(Clone)]
pub struct DataImpl(Arc<dyn DataInterface>);

impl DataImpl {
    pub fn new<D: DataInterface + 'static>(data: D) -> DataImpl {
        DataImpl(Arc::new(data))
    }
}

impl Deref for DataImpl {
    type Target = dyn DataInterface + 'static;
    fn deref(&self) -> &(dyn DataInterface + 'static) {
        self.0.deref()
    }
}

/// Dynamic dispatch events operations to a backend-specific implementation.
#[derive(Clone)]
pub struct EventsImpl(Arc<dyn EventsInterface>);

impl EventsImpl {
    pub fn new<E: EventsInterface + 'static>(events: E) -> EventsImpl {
        EventsImpl(Arc::new(events))
    }
}

impl Deref for EventsImpl {
    type Target = dyn EventsInterface + 'static;
    fn deref(&self) -> &(dyn EventsInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of model persist operations.
///
/// See `store::persist::Persist` for descriptions of methods.
pub trait PersistInterface: Send + Sync {
    fn event(&self, event: Event, span: Option<SpanContext>) -> Result<()>;
}

/// Dynamic dispatch persist operations to a backend-specific implementation.
#[derive(Clone)]
pub struct PersistImpl(Arc<dyn PersistInterface>);

impl PersistImpl {
    pub fn new<P: PersistInterface + 'static>(persist: P) -> PersistImpl {
        PersistImpl(Arc::new(persist))
    }
}

impl Deref for PersistImpl {
    type Target = dyn PersistInterface + 'static;
    fn deref(&self) -> &(dyn PersistInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of top level store operations.
///
/// Mainly a way to return interfaces to grouped store operations.
///
/// See `store::Store` for descriptions of methods.
pub trait StoreInterface: Send + Sync {
    fn events(&self) -> EventsImpl;
    fn persist(&self) -> PersistImpl;
}

/// Dynamic dispatch all operations to a backend-specific implementation.
#[derive(Clone)]
pub struct StoreImpl(Arc<dyn StoreInterface>);

impl StoreImpl {
    pub fn new<S: StoreInterface + 'static>(store: S) -> StoreImpl {
        StoreImpl(Arc::new(store))
    }
}

impl Deref for StoreImpl {
    type Target = dyn StoreInterface + 'static;
    fn deref(&self) -> &(dyn StoreInterface + 'static) {
        self.0.deref()
    }
}

/// Definition of supported validation operations.
///
/// See `admin::validate::Validate` for descriptions of methods.
pub trait ValidateInterface: Send + Sync {
    fn indexes(&self) -> Result<Vec<ValidationResult>>;
    fn removed_entities(&self) -> Result<Vec<ValidationResult>>;
    fn schema(&self) -> Result<Vec<ValidationResult>>;
}

/// Dynamic dispatch validate operations to a backend-specific implementation.
#[derive(Clone)]
pub struct ValidateImpl(Arc<dyn ValidateInterface>);

impl ValidateImpl {
    pub fn new<V: ValidateInterface + 'static>(validate: V) -> ValidateImpl {
        ValidateImpl(Arc::new(validate))
    }
}

impl Deref for ValidateImpl {
    type Target = dyn ValidateInterface + 'static;
    fn deref(&self) -> &(dyn ValidateInterface + 'static) {
        self.0.deref()
    }
}
