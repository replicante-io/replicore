use std::time::Duration;

use anyhow::Result;

use replicore_context::Context;
use replicore_events::emit::EventsFixture;

use super::Authorisation;
use super::Authoriser;
use crate::AuthContext;
use crate::Entity;
use crate::EntitySystem;

const ONE_SEC: Duration = Duration::from_secs(1);

/// Test Authorisation backend to allow all requests.
struct AllowAll;

#[async_trait::async_trait]
impl Authorisation for AllowAll {
    async fn authorise(&self, _: &Context) -> Result<()> {
        Ok(())
    }
}

/// Test Authorisation backend to deny all requests.
struct DenyAll;

#[async_trait::async_trait]
impl Authorisation for DenyAll {
    async fn authorise(&self, context: &Context) -> Result<()> {
        let auth = context.auth.as_ref().unwrap();
        let forbid = super::Forbidden::from(auth);
        anyhow::bail!(forbid)
    }
}

fn auth_context() -> AuthContext {
    AuthContext {
        action: crate::Action::define("test", "noop"),
        entity: crate::Entity::Anonymous,
        impersonate: None,
        resource: crate::Resource {
            kind: "test".to_string(),
            metadata: Default::default(),
            resource_id: "noop".to_string(),
        },
    }
}

#[tokio::test]
async fn bypass_backend_for_system_entities() {
    let mut events = EventsFixture::new();
    let mut auth = auth_context();
    auth.entity = Entity::System(EntitySystem {
        component: "test".into(),
    });
    let context = Context::fixture().derive().authenticated(auth).build();

    let auth = Authoriser::wrap(DenyAll, events.backend().into());
    auth.authorise(&context)
        .await
        .expect("request should be authorised");

    let audit = events.pop_audit_timeout(ONE_SEC).await.unwrap();
    assert_eq!(audit.code, super::AUDIT_AUTHORISATION);
    assert_eq!(
        audit.payload,
        serde_json::json!({
            "action": "test:noop",
            "decision": "Allow",
            "entity": {
                "component": "test",
                "kind": "system",
            },
            "resource": {
                "kind": "test",
                "metadata": {},
                "resource_id": "noop",
            },
            "trace_id": null,
        })
    );
}

#[tokio::test]
async fn audit_authorisation_request() {
    let mut events = EventsFixture::new();
    let context = Context::fixture()
        .derive()
        .authenticated(auth_context())
        .build();

    let auth = Authoriser::wrap(AllowAll, events.backend().into());
    auth.authorise(&context)
        .await
        .expect("request to be authorised");

    let audit = events.pop_audit_timeout(ONE_SEC).await.unwrap();
    assert_eq!(audit.code, super::AUDIT_AUTHORISATION);
    assert_eq!(
        audit.payload,
        serde_json::json!({
            "action": "test:noop",
            "decision": "Allow",
            "entity": {
                "kind": "anonymous",
            },
            "resource": {
                "kind": "test",
                "metadata": {},
                "resource_id": "noop",
            },
            "trace_id": null,
        })
    );
}

#[tokio::test]
#[should_panic(expected = "cannot authorise without an auth context")]
async fn panic_without_auth_context() {
    let events = EventsFixture::new();
    let auth = Authoriser::wrap(AllowAll, events.backend().into());
    let context = Context::fixture();
    let _ = auth.authorise(&context).await;
}

#[tokio::test]
async fn request_allowed() {
    let events = EventsFixture::new();
    let auth = Authoriser::wrap(AllowAll, events.backend().into());
    let context = Context::fixture()
        .derive()
        .authenticated(auth_context())
        .build();
    auth.authorise(&context)
        .await
        .expect("request to be authorised");
}

#[tokio::test]
async fn request_denied() {
    let events = EventsFixture::new();
    let auth = Authoriser::wrap(DenyAll, events.backend().into());
    let context = Context::fixture()
        .derive()
        .authenticated(auth_context())
        .build();
    assert!(auth
        .authorise(&context)
        .await
        .unwrap_err()
        .is::<super::Forbidden>());
}
