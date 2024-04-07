//! Orchestrator Action reusable logic.
use anyhow::Result;

use replisdk::core::models::api::OActionSpec;
use replisdk::core::models::oaction::OAction;
use replisdk::core::models::oaction::OActionRef;

use replicore_context::Context;
use replicore_events::Event;
use replicore_store::query::LookupOAction;

use super::CoreSDK;

impl CoreSDK {
    /// Create a new [`OAction`] record ensuring all needed attributes are set.
    pub async fn oaction_create(&self, context: &Context, spec: OActionSpec) -> Result<OActionRef> {
        // If an Action ID is given ensure it does not exist.
        if let Some(action_id) = spec.action_id {
            let query = LookupOAction::by(&spec.ns_id, &spec.cluster_id, action_id);
            let oaction = self.injector.store.query(&context, query).await?;
            if oaction.is_some() {
                let error = crate::errors::ActionExists {
                    ns_id: spec.ns_id.clone(),
                    cluster_id: spec.cluster_id.clone(),
                    action_id,
                };
                anyhow::bail!(error);
            }
        }

        // Expand the spec into a full object.
        let action_id = spec.action_id.unwrap_or_else(uuid::Uuid::new_v4);
        let action_ref = OActionRef {
            ns_id: spec.ns_id.clone(),
            cluster_id: spec.cluster_id.clone(),
            action_id,
        };
        let oaction = OAction {
            ns_id: spec.ns_id,
            cluster_id: spec.cluster_id,
            action_id,
            args: spec.args,
            created_ts: time::OffsetDateTime::now_utc(),
            finished_ts: None,
            kind: spec.kind,
            metadata: spec.metadata,
            scheduled_ts: None,
            state: spec.approval.into(),
            state_payload: None,
            state_payload_error: None,
            timeout: spec.timeout,
        };

        // Apply the cluster spec.
        let event = Event::new_with_payload(crate::constants::OACTION_CREATE, &oaction)?;
        self.injector.events.change(context, event).await?;
        self.injector.store.persist(context, oaction).await?;
        Ok(action_ref)
    }
}
