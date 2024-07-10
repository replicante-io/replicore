//! Node Action reusable logic.
use anyhow::Result;

use replisdk::core::models::api::NActionSpec;
use replisdk::core::models::naction::NAction;
use replisdk::core::models::naction::NActionPhase;
use replisdk::core::models::naction::NActionRef;
use replisdk::core::models::naction::NActionState;

use replicore_context::Context;
use replicore_events::Event;
use replicore_store::query::LookupNAction;

use super::CoreSDK;

impl CoreSDK {
    /// Approve a [`NAction`] record for scheduling ensuring appropriate events are emitted.
    pub async fn naction_approve(&self, context: &Context, mut action: NAction) -> Result<()> {
        action.state.phase = NActionPhase::PendingSchedule;
        let event = Event::new_with_payload(crate::constants::NACTION_APPROVE, &action)?;
        self.injector.events.change(context, event).await?;
        self.injector.store.persist(context, action).await?;
        Ok(())
    }

    /// Cancel a [`NAction`] and prevent any further execution.
    pub async fn naction_cancel(&self, context: &Context, mut action: NAction) -> Result<()> {
        action.finish(NActionPhase::Cancelled);
        let event = Event::new_with_payload(crate::constants::NACTION_CANCEL, &action)?;
        self.injector.events.change(context, event).await?;
        self.injector.store.persist(context, action).await?;
        Ok(())
    }

    /// Create a new [`NAction`] record ensuring all needed attributes are set.
    pub async fn naction_create(&self, context: &Context, spec: NActionSpec) -> Result<NActionRef> {
        // If an Action ID is given ensure it does not exist.
        if let Some(action_id) = spec.action_id {
            let query = LookupNAction::by(&spec.ns_id, &spec.cluster_id, &spec.node_id, action_id);
            let action = self.injector.store.query(context, query).await?;
            if action.is_some() {
                let error = crate::errors::NActionExists {
                    ns_id: spec.ns_id,
                    cluster_id: spec.cluster_id,
                    node_id: spec.node_id,
                    action_id,
                };
                anyhow::bail!(error);
            }
        }

        // Expand the spec into a full object.
        let action_id = spec.action_id.unwrap_or_else(uuid::Uuid::new_v4);
        let action_ref = NActionRef {
            ns_id: spec.ns_id.clone(),
            cluster_id: spec.cluster_id.clone(),
            node_id: spec.node_id.clone(),
            action_id,
        };
        let action = NAction {
            ns_id: spec.ns_id,
            cluster_id: spec.cluster_id,
            node_id: spec.node_id,
            action_id,
            args: spec.args,
            created_time: time::OffsetDateTime::now_utc(),
            finished_time: None,
            kind: spec.kind,
            metadata: spec.metadata,
            scheduled_time: None,
            state: NActionState {
                error: None,
                payload: None,
                phase: spec.approval.into(),
            },
        };

        // Apply the cluster spec.
        let event = Event::new_with_payload(crate::constants::NACTION_CREATE, &action)?;
        self.injector.events.change(context, event).await?;
        self.injector.store.persist(context, action).await?;
        Ok(action_ref)
    }

    /// Reject a [`NAction`] record to prevent scheduling.
    pub async fn naction_reject(&self, context: &Context, mut action: NAction) -> Result<()> {
        action.finish(NActionPhase::Cancelled);
        let event = Event::new_with_payload(crate::constants::NACTION_REJECT, &action)?;
        self.injector.events.change(context, event).await?;
        self.injector.store.persist(context, action).await?;
        Ok(())
    }
}
