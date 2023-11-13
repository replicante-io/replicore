//! Module to deal with the Authentication (who is accessing) side of Auth.
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as Json;

use replicore_context::Context;

use super::Entity;

/// Operations implemented by Authentication mechanisms and services supported by Replicante Core.
#[async_trait::async_trait]
pub trait Authentication: Send + Sync {
    /// Determine the [`Entity`] performing a request and verify their identity is valid.
    ///
    /// [`Authentication`] implementations must respect the following expectations:
    ///
    /// - Determine the identity of the [`Entity`] attached to a request, including impersonation.
    /// - Ensure the identity information can be trusted and has not been tempered with.
    /// - If identity information is not part or the request return an [`Entity::Anonymous`].
    /// - If identity information is part of the request but not valid return an appropriate error.
    async fn authenticate(
        &self,
        context: &Context,
        transport: &dyn IdentityReader,
    ) -> Result<Entity>;
}

/// Initialisation logic for [`Authentication`] implementations.
#[async_trait::async_trait]
pub trait AuthenticationFactory: Send + Sync {
    /// Validate the user provided configuration for the backend.
    fn conf_check(&self, context: &Context, conf: &Json) -> Result<()>;

    /// Register backend specific metrics.
    fn register_metrics(&self, registry: &prometheus::Registry) -> Result<()>;

    /// Initialise an [`Authenticator`] object.
    async fn authenticator<'a>(&self, args: AuthenticationFactoryArgs<'a>)
        -> Result<Authenticator>;
}

/// Arguments passed to the [`AuthenticationFactory`] initialisation method.
pub struct AuthenticationFactoryArgs<'a> {
    /// The configuration block for the backend to initialise.
    pub conf: &'a Json,
}

/// Determine the [`Entity`] requesting actions in a trusted way.
#[derive(Clone)]
pub struct Authenticator {
    /// Authentication backend to determine the [`Entity`] with.
    inner: Arc<dyn Authentication>,
}

impl Authenticator {
    /// Determine the [`Entity`] performing a request and verify their identity is valid.
    ///
    /// For details see [`Authentication::authenticate`].
    pub async fn authenticate(
        &self,
        context: &Context,
        transport: &dyn IdentityReader,
    ) -> Result<Entity> {
        self.inner.authenticate(context, transport).await
    }
}

impl<T> From<T> for Authenticator
where
    T: Authentication + 'static,
{
    fn from(value: T) -> Self {
        let inner = Arc::new(value);
        Authenticator { inner }
    }
}

/// Read identity information to discover and verify [`Entity`]s from a variety of sources.
pub trait IdentityReader {
    /// Look for a metadata value with the given key.
    ///
    /// Returns `None` if the entry is missing or an `Err` if the metadata could
    /// not be read or decoded.
    ///
    /// For example in HTTP(S) requests metadata should be extracted from headers.
    fn metadata(&self, name: &str) -> Result<Option<&str>>;
}

#[cfg(feature = "actix-web")]
impl IdentityReader for actix_web::HttpRequest {
    fn metadata(&self, name: &str) -> Result<Option<&str>> {
        match self.headers().get(name) {
            None => Ok(None),
            Some(header) => {
                let value = header.to_str()?;
                Ok(Some(value))
            }
        }
    }
}
