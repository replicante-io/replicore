//! Unit test to ensure Store interface type conversions work nicely.
use replisdk::core::models::namespace::Namespace;

use replicore_context::Context;

use crate::query::LookupNamespace;
use crate::Store;

#[tokio::test]
async fn check_delete_interface() {
    let context = Context::fixture();
    let namespace = Namespace { id: "test".into() };
    let store = Store::fixture();
    store
        .delete(&context, &namespace)
        .await
        .expect("namespace delete to be ok");
}

#[tokio::test]
async fn check_query_interface() {
    let context = Context::fixture();
    let lookup = LookupNamespace { id: "test".into() };
    let store = Store::fixture();
    store
        .query(&context, lookup)
        .await
        .expect("namespace query to be ok");
}

#[tokio::test]
async fn check_persist_interface() {
    let context = Context::fixture();
    let namespace = Namespace { id: "test".into() };
    let store = Store::fixture();
    store
        .persist(&context, namespace)
        .await
        .expect("namespace persist to be ok");
}
