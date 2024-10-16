//! Unit test to ensure Store interface type conversions work nicely.
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::namespace::NamespaceStatus;

use replicore_context::Context;

use crate::ids::NamespaceID;
use crate::query::LookupNamespace;
use crate::Store;

fn mock_namespace() -> Namespace {
    Namespace {
        id: "test".into(),
        tls: Default::default(),
        settings: Default::default(),
        status: NamespaceStatus::Active,
    }
}

#[tokio::test]
async fn check_delete_interface() {
    let context = Context::fixture();
    let namespace = mock_namespace();
    let store = Store::fixture();
    store
        .delete(&context, &namespace)
        .await
        .expect("namespace delete to be ok");
}

#[tokio::test]
async fn check_query_interface() {
    let context = Context::fixture();
    let lookup = NamespaceID { id: "test".into() };
    let lookup = LookupNamespace(lookup);
    let store = Store::fixture();
    store
        .query(&context, lookup)
        .await
        .expect("namespace query to be ok");
}

#[tokio::test]
async fn check_persist_interface() {
    let context = Context::fixture();
    let namespace = mock_namespace();
    let store = Store::fixture();
    store
        .persist(&context, namespace)
        .await
        .expect("namespace persist to be ok");
}
