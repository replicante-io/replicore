use opentracingrust::SpanContext;

use replicante_models_core::actions::node::Action;
use replicante_models_core::actions::node::ActionHistory;
use replicante_models_core::cluster::OrchestrateReport;
use replicante_models_core::events::Event;

use crate::backend::PersistImpl;
use crate::Result;

/// Persist (insert or update) models to the store.
pub struct Persist {
    persist: PersistImpl,
}

impl Persist {
    pub(crate) fn new(persist: PersistImpl) -> Persist {
        Persist { persist }
    }

    /// Create or update an `Action` record.
    pub fn action<S>(&self, action: Action, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.action(action, span.into())
    }

    /// Append the given `ActionHistory`s to the transition records.
    pub fn action_history<S>(&self, history: Vec<ActionHistory>, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.action_history(history, span.into())
    }

    /// Create or update a cluster `OrchestrateReport` record.
    pub fn cluster_orchestrate_report<S>(&self, report: OrchestrateReport, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.cluster_orchestrate_report(report, span.into())
    }

    /// Create or update an `Event` record.
    pub fn event<S>(&self, event: Event, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.persist.event(event, span.into())
    }
}
