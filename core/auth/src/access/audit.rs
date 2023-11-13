//! Information attached to authorisation audit events.
use anyhow::Result;
use opentelemetry_api::trace::TraceContextExt;
use opentelemetry_api::trace::TraceId;
use opentelemetry_api::Context as OTelContext;
use serde::Deserialize;
use serde::Serialize;

use replisdk::core::models::auth::AuthContext;

use replicore_events::Event;

use super::Forbidden;
use crate::Action;
use crate::Entity;
use crate::Resource;

/// Event code for audit authorisation events.
pub const AUDIT_AUTHORISATION: &str = "AUDIT_AUTHORISATION";

/// Payload for Audit events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Audit {
    /// Action being authorised.
    pub action: Action,

    /// Result of the authorisation process.
    pub decision: AuditDecision,

    /// Entity the action is performed by.
    pub entity: Entity,

    /// Resource the action is performed on.
    pub resource: Resource,

    /// Tracing ID to link this audit event to a larger context, if tracing is available.
    pub trace_id: Option<String>,
}

impl Audit {
    /// Compose an authorisation audit event from authorisation information.
    pub fn event(context: &AuthContext, result: &Result<()>) -> Result<Event> {
        let trace_id = OTelContext::current().span().span_context().trace_id();
        let trace_id = if trace_id == TraceId::INVALID {
            None
        } else {
            Some(trace_id.to_string())
        };
        let payload = Audit {
            action: context.action.clone(),
            decision: AuditDecision::from(result),
            entity: context.entity.clone(),
            resource: context.resource.clone(),
            trace_id,
        };
        Event::new_with_payload(AUDIT_AUTHORISATION, payload)
    }
}

/// Decision of an authorisation request reported in an [`Audit`] event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuditDecision {
    /// The request was authorised.
    Allow,

    /// The request was denied.
    Deny,

    /// There was an error performing the authorisation check (so the request was denied).
    Error,
}

impl From<&Result<()>> for AuditDecision {
    fn from(value: &Result<()>) -> Self {
        match value {
            Ok(()) => AuditDecision::Allow,
            Err(error) if error.is::<Forbidden>() => AuditDecision::Deny,
            Err(_) => AuditDecision::Deny,
        }
    }
}
