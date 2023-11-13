//! Insecure Authentication and Authorisation service to allow all access.
//!
//! This backend is intended for early development cycles or demo instances.
use anyhow::Result;
use serde_json::Value as Json;

use replicore_auth::access::Authorisation;
use replicore_auth::access::AuthorisationFactory;
use replicore_auth::access::AuthorisationFactoryArgs;
use replicore_auth::access::Authoriser;
use replicore_auth::identity::Authentication;
use replicore_auth::identity::AuthenticationFactory;
use replicore_auth::identity::AuthenticationFactoryArgs;
use replicore_auth::identity::Authenticator;
use replicore_auth::identity::IdentityReader;
use replicore_auth::Entity;
use replicore_context::Context;

/// Unconditionally authenticate all requested as [`Entity::Anonymous`].
pub struct Anonymous;

#[async_trait::async_trait]
impl Authentication for Anonymous {
    async fn authenticate(&self, _: &Context, _: &dyn IdentityReader) -> Result<Entity> {
        Ok(Entity::Anonymous)
    }
}

#[async_trait::async_trait]
impl AuthenticationFactory for Anonymous {
    fn conf_check(&self, _: &Context, _: &Json) -> Result<()> {
        Ok(())
    }

    fn register_metrics(&self, _: &prometheus::Registry) -> Result<()> {
        Ok(())
    }

    async fn authenticator<'a>(&self, _: AuthenticationFactoryArgs<'a>) -> Result<Authenticator> {
        Ok(Authenticator::from(Anonymous))
    }
}

/// Authorise unrestricted access for all requests.
pub struct Unrestricted;

#[async_trait::async_trait]
impl Authorisation for Unrestricted {
    async fn authorise(&self, _: &Context) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl AuthorisationFactory for Unrestricted {
    fn conf_check(&self, _: &Context, _: &Json) -> Result<()> {
        Ok(())
    }

    fn register_metrics(&self, _: &prometheus::Registry) -> Result<()> {
        Ok(())
    }

    async fn authoriser<'a>(&self, args: AuthorisationFactoryArgs<'a>) -> Result<Authoriser> {
        Ok(Authoriser::wrap(Unrestricted, args.events.clone()))
    }
}
