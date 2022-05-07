use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use slog::Logger;
use uuid::Uuid;

use replicante_externals_mongodb::admin::ValidationResult;
use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::admin::Version;
use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::events::Event;
use replicante_service_healthcheck::HealthChecks;

use crate::store::actions::SearchFilters as ActionsSearchFilters;
use crate::store::cluster::ClusterAttributes;
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

/// Instantiate a new storage admin backend based on the given configuration.
pub fn backend_factory_admin(config: Config, logger: Logger) -> Result<AdminImpl> {
    let admin = match config {
        Config::MongoDB(config) => AdminImpl::new(self::mongo::Admin::make(config, logger)?),
    };
    Ok(admin)
}

// Macro definition to generate an interface trait with a wrapping wrapper
// for dynamic dispatch to Send + Sync + 'static implementations.
macro_rules! arc_interface {
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name: Send + Sync $trait_def

        $(#[$struct_meta])*
        #[derive(Clone)]
        pub struct $struct_name(Arc<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Arc::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }
    }
}
macro_rules! box_interface {
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name $trait_def

        $(#[$struct_meta])*
        pub struct $struct_name(Box<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Box::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut (dyn $trait_name + 'static) {
                self.0.deref_mut()
            }
        }
    };
}

arc_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct AdminImpl,

    /// Definition of top level store administration operations.
    ///
    /// Mainly a way to return interfaces to grouped store operations.
    ///
    /// See `admin::Admin` for descriptions of methods.
    trait AdminInterface,

    interface {
        fn data(&self) -> DataImpl;
        fn validate(&self) -> ValidateImpl;
        fn version(&self) -> Result<Version>;
    }
}

arc_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct StoreImpl,

    /// Definition of top level store operations.
    ///
    /// Mainly a way to return interfaces to grouped store operations.
    ///
    /// See `store::Store` for descriptions of methods.
    trait StoreInterface,

    interface {
        fn actions(&self, cluster_id: String) -> ActionsImpl;
        fn cluster(&self) -> ClusterImpl;
        fn events(&self) -> EventsImpl;
        fn persist(&self) -> PersistImpl;
    }
}

box_interface! {
    /// Dynamic dispatch actions operations to a backend-specific implementation.
    struct ActionsImpl,

    /// Definition of actions operations.
    ///
    /// See `store::actions::Actions` for descriptions of methods.
    trait ActionsInterface,

    interface {
        fn action(&self, action_id: Uuid, span: Option<SpanContext>) -> Result<Option<Action>>;
        fn finish_history(
            &self,
            action_id: Uuid,
            finished_ts: DateTime<Utc>,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn history(
            &self,
            action_id: Uuid,
            span: Option<SpanContext>,
        ) -> Result<Vec<ActionHistory>>;
        fn search(
            &self,
            filters: ActionsSearchFilters,
            span: Option<SpanContext>,
        ) -> Result<Cursor<Action>>;
    }
}

box_interface! {
    /// Dynamic dispatch cluster operations to a backend-specific implementation.
    struct ClusterImpl,

    /// Definition of cluster operations.
    ///
    /// See `store::cluster::Cluster` for descriptions of methods.
    trait ClusterInterface,

    interface {
        fn orchestrate_report(
            &self,
            attrs: &ClusterAttributes,
            span: Option<SpanContext>,
        ) -> Result<Option<OrchestrateReport>>;
    }
}

box_interface! {
    /// Dynamic dispatch events operations to a backend-specific implementation.
    struct EventsImpl,

    /// Definition of events operations.
    ///
    /// See `store::events::Events` for descriptions of methods.
    trait EventsInterface,

    interface {
        fn range(
            &self,
            filters: EventsFilters,
            options: EventsOptions,
            span: Option<SpanContext>,
        ) -> Result<Cursor<Event>>;
    }
}

box_interface! {
    /// Dynamic dispatch all data admin operations to a backend-specific implementation.
    struct DataImpl,

    /// Definition of supported data admin operations.
    ///
    /// See `admin::data::Data` for descriptions of methods.
    trait DataInterface,

    interface {
        fn actions(&self) -> Result<Cursor<Action>>;
        fn actions_history(&self) -> Result<Cursor<ActionHistory>>;
        fn events(&self) -> Result<Cursor<Event>>;
    }
}

box_interface! {
    /// Dynamic dispatch persist operations to a backend-specific implementation.
    struct PersistImpl,

    /// Definition of model persist operations.
    ///
    /// See `store::persist::Persist` for descriptions of methods.
    trait PersistInterface,

    interface {
        fn action(
            &self,
            action: Action,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn action_history(
            &self,
            history: Vec<ActionHistory>,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn cluster_orchestrate_report(
            &self,
            report: OrchestrateReport,
            span: Option<SpanContext>,
        ) -> Result<()>;
        fn event(&self, event: Event, span: Option<SpanContext>) -> Result<()>;
    }
}

box_interface! {
    /// Dynamic dispatch validate operations to a backend-specific implementation.
    struct ValidateImpl,

    /// Definition of supported validation operations.
    ///
    /// See `admin::validate::Validate` for descriptions of methods.
    trait ValidateInterface,

    interface {
        fn removed_entities(&self) -> Result<Vec<ValidationResult>>;
        fn schema(&self) -> Result<Vec<ValidationResult>>;
    }
}
