//! Module to deal with the Authorisation (what can be done) side of Auth.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;
use replicore_events::emit::Events;

mod audit;

#[cfg(test)]
mod test;

pub use self::audit::Audit;
pub use self::audit::AuditDecision;
pub use self::audit::AUDIT_AUTHORISATION;

use super::AuthContext;
use crate::Entity;

/// Operations implemented by Authorisation modes and services supported by Replicante Core.
#[async_trait::async_trait]
pub trait Authorisation: Send + Sync {
    /// Determine if a request should be authorised based on the information in its [`Context`].
    ///
    /// Authorisation implementations will use the given [`Context::auth`] information
    /// to check who is trying to do what on which resource.
    ///
    /// The return value indicates if authorisation was granted:
    ///
    /// - Return `Ok` if the request is authorised.
    /// - Return `Err` with a [`Forbidden`] error if the request was denied.
    /// - Return `Err` with any other error if the authorisation check failed
    ///   (this will still deny requests).
    ///
    /// This approach makes the code checking for access clear and concise.
    ///
    /// NOTE: thanks to the use of the [`anyhow`] crate errors returned by this method
    /// can be "marked" using context methods.
    /// This pattern can be used to report an authorisation failure while also preserving
    /// additional evaluation information:
    ///
    /// ```ignore
    /// use anyhow::Context;
    ///
    /// let error = anyhow::anyhow!("example denied request");
    /// Err(error.context(Forbidden))
    /// ```
    ///
    /// ## Panics
    ///
    /// This method can expect [`Context::auth`] to be `Some` and should panic if it is not.
    /// Refer to [`Authoriser::authorise`] for an explanation of why.
    async fn authorise(&self, context: &Context) -> Result<()>;
}

/// Initialisation logic for [`Authorisation`] implementations.
#[async_trait::async_trait]
pub trait AuthorisationFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Initialise an [`Authoriser`] object.
    async fn authoriser<'a>(&self, args: AuthorisationFactoryArgs<'a>) -> Result<Authoriser>;
}

/// Arguments passed to the [`AuthorisationFactory`] initialisation method.
pub struct AuthorisationFactoryArgs<'a> {
    /// The configuration block for the backend to initialise.
    pub conf: &'a Json,

    /// Interface to the events streaming service to emit (audit) events.
    pub events: &'a Events,
}

/// Verify permissions for a requesting entity to perform a specific action on a resource.
#[derive(Clone)]
pub struct Authoriser {
    /// Interface to the events streaming service to emit (audit) events.
    events: Events,

    /// Authorisation backend to check for permission.
    inner: Arc<dyn Authorisation>,
}

impl Authoriser {
    /// Verify a [`Context`] for correct authorisation with a supported backend.
    ///
    /// The [`Context::auth`] contains the information used to determine authorisation
    /// of the action by the requesting entity.
    /// If authorisation is denied the method returns a [`Forbidden`] error.
    ///
    /// After authorisation is verified this method also emits an audit event
    /// for system operators to troubleshoot access problems or verify past access.
    ///
    /// ## System [`Entity`](super::Entity) bypass
    ///
    /// The `system` family of [`Entity`]es is used to represent actions initiated by
    /// the Control Plane itself to perform duties.
    ///
    /// To ensure authorisation is not missed by mistake all requests must be authorised
    /// at some point, even those started internally by the control plane.
    ///
    /// Because these requests are key to correct operations and to avoid
    /// complexity or mistakes in authorisation policies any request initiated by the
    /// Control Plane is allowed without checking with the configured backend.
    /// An audit event is still logged to ensure a complete picture of what is happening.
    ///
    /// ## Panics
    ///
    /// This method can expect [`Context::auth`] to be `Some` and will panic if it is not.
    ///
    /// While this pattern introduces a panic that could be avoided with the help of the type
    /// system it also ensures that checking for authorisation:
    ///
    /// - Is easy to perform unconditionally during operations.
    /// - Ensures that cases where authorisation attempts are performed before authentication
    ///   is completed don't result in incorrect authorisation and can be identified quickly.
    pub async fn authorise(&self, context: &Context) -> Result<()> {
        // An authorisation context is required.
        if context.auth.is_none() {
            panic!("cannot authorise without an auth context");
        }

        // Check with the backend for all non-system entities.
        let entity = &context.auth.as_ref().unwrap().entity;
        let result = if matches!(entity, Entity::System(_)) {
            Ok(())
        } else {
            self.inner.authorise(context).await
        };

        // Audit the authorisation result and done.
        self.audit(context, &result).await;
        result
    }

    /// Wrap an [`Authorisation`] interface for use by the system.
    pub fn wrap<T>(inner: T, events: Events) -> Self
    where
        T: Authorisation + 'static,
    {
        let inner = Arc::new(inner);
        Authoriser { events, inner }
    }
}

impl Authoriser {
    /// Generate an authorisation audit event and emit it.
    ///
    /// Errors during audit are ignored to preserve availability in case of
    /// upstream issues or misconfiguration.
    ///
    /// NOTE:
    ///   Only auditing errors are ignored, authorisation errors will prevent access.
    ///   This is done to ensure service and data protection over availability.
    async fn audit(&self, context: &Context, result: &Result<()>) {
        let auth = context.auth.as_ref().expect("Context::auth must be set");
        let event = match Audit::event(auth, result) {
            Ok(event) => event,
            Err(error) => {
                slog::error!(
                    context.logger,
                    "Failed to JSON serialise authorisation audit event payload";
                    "audit" => true,
                    replisdk::utils::error::slog::ErrorAttributes::from(&error),
                );
                return;
            }
        };
        if let Err(error) = self.events.audit(context, event).await {
            slog::error!(
                context.logger,
                "Failed to emit authorisation audit event";
                "audit" => true,
                replisdk::utils::error::slog::ErrorAttributes::from(&error),
            );
        }
    }
}

/// An entity is not allowed to perform an action on a resource.
#[derive(Debug, thiserror::Error)]
#[error("entity \"{entity}\" is not allowed to perform \"{action}\" on resource \"{resource}\"")]
pub struct Forbidden {
    action: String,
    entity: String,
    resource: String,
}

impl Forbidden {
    /// Deny an entity from performing an action onto a resource.
    pub fn deny<S1, S2, S3>(entity: S1, action: S2, resource: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        Self {
            action: action.into(),
            entity: entity.into(),
            resource: resource.into(),
        }
    }
}

impl From<&AuthContext> for Forbidden {
    fn from(value: &AuthContext) -> Self {
        let action = value.action.clone().into();
        let entity = value
            .impersonate
            .as_ref()
            .map(|entity| entity.to_string())
            .unwrap_or_else(|| value.entity.to_string());

        let kind = &value.resource.kind;
        let resource_id = &value.resource.resource_id;
        let ns = value.resource.metadata.get(super::RESOURCE_NAMESPACE);
        let resource = if let Some(ns) = ns {
            format!("{}/{}.{}", kind, ns, resource_id)
        } else {
            format!("{}/{}", kind, resource_id)
        };

        Self {
            action,
            entity,
            resource,
        }
    }
}
