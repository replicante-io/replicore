use std::time::Duration;

use anyhow::Result;
use serde_json::Value;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::namespace::Namespace;
use replisdk::core::models::namespace::NamespaceStatus;
use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformTransport;
use replisdk::core::models::platform::PlatformTransportUrl;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_context::Context;
use replicore_errors::NamespaceNotActive;
use replicore_errors::NamespaceNotFound;
use replicore_injector::Injector;
use replicore_injector::InjectorFixture;
use replicore_store::query::LookupClusterDiscovery;
use repliplatform_client::Client;

use super::discover::discover;
use crate::callback::Callback;
use crate::errors::PlatformNotActive;
use crate::errors::PlatformNotFound;
use crate::DiscoverPlatform;

/// Factory to return new unit test clients.
struct UnittestClientFactory;

#[async_trait::async_trait]
impl replicore_clients_platform::UrlFactory for UnittestClientFactory {
    async fn init(&self, _: &Context, _: &PlatformTransportUrl) -> Result<Client> {
        let client = repliplatform_client::fixture::Client::default();
        client.append_node("cluster1", "node1");
        client.append_node("cluster1", "node2");
        client.append_node("cluster1", "node3");
        client.append_node("cluster2", "node4");
        Ok(Client::from(client))
    }
}

/// Initialise a clients factory with unit test clients.
async fn fixed_callback() -> (Callback, InjectorFixture) {
    let mut clients = replicore_clients_platform::PlatformClients::empty();
    clients.with_url_factory("unittest", UnittestClientFactory);
    let mut injector_fixture = Injector::fixture();
    injector_fixture.injector.clients.platform = clients;
    fixed_db(&injector_fixture.injector).await;
    let callback = Callback {
        injector: injector_fixture.injector.clone(),
    };
    (callback, injector_fixture)
}

/// Populate injected DB for tests.
async fn fixed_db(injector: &Injector) {
    // Inactive and active namespaces.
    let ns = Namespace {
        id: "test".into(),
        tls: Default::default(),
        settings: Default::default(),
        status: NamespaceStatus::Inactive,
    };
    injector.store.persist(&injector.context, ns).await.unwrap();
    let ns = Namespace {
        id: "default".into(),
        tls: Default::default(),
        settings: Default::default(),
        status: NamespaceStatus::Active,
    };
    injector.store.persist(&injector.context, ns).await.unwrap();

    // Inactive and active platform.
    let platform = Platform {
        ns_id: "default".into(),
        name: "test".into(),
        active: false,
        discovery: Default::default(),
        transport: PlatformTransport::Url(PlatformTransportUrl {
            base_url: "http://localhost:1234".into(),
            tls_ca_bundle: None,
            tls_insecure_skip_verify: false,
        }),
    };
    injector
        .store
        .persist(&injector.context, platform)
        .await
        .unwrap();
    let platform = Platform {
        ns_id: "default".into(),
        name: "unit".into(),
        active: true,
        discovery: Default::default(),
        transport: PlatformTransport::Url(PlatformTransportUrl {
            base_url: "unittest://localhost:1234".into(),
            tls_ca_bundle: None,
            tls_insecure_skip_verify: false,
        }),
    };
    injector
        .store
        .persist(&injector.context, platform)
        .await
        .unwrap();

    // Insert known cluster records to test upsert logic.
    let discovery = ClusterDiscovery {
        ns_id: "default".into(),
        cluster_id: "cluster2".into(),
        nodes: vec![ClusterDiscoveryNode {
            node_id: "node5".into(),
            node_class: "unit".into(),
            agent_address: "unittest://node5".into(),
            node_group: None,
        }],
    };
    injector
        .store
        .persist(&injector.context, discovery)
        .await
        .unwrap();
}

#[tokio::test]
async fn ns_missing() {
    let (callback, _) = fixed_callback().await;
    let context = callback.injector.context.clone();
    let request = DiscoverPlatform::new("missing", "missing");
    let result = discover(&context, &callback, request).await;
    match result {
        Ok(()) => panic!("discovery expected to fail"),
        Err(error) if error.is::<NamespaceNotFound>() => (),
        Err(error) => panic!("discovery failed with unexpected error: {:?}", error),
    }
}

#[tokio::test]
async fn ns_not_active() {
    let (callback, _) = fixed_callback().await;
    let context = callback.injector.context.clone();
    let request = DiscoverPlatform::new("test", "missing");
    let result = discover(&context, &callback, request).await;
    match result {
        Ok(()) => panic!("discovery expected to fail"),
        Err(error) if error.is::<NamespaceNotActive>() => (),
        Err(error) => panic!("discovery failed with unexpected error: {:?}", error),
    }
}

#[tokio::test]
async fn platform_missing() {
    let (callback, _) = fixed_callback().await;
    let context = callback.injector.context.clone();
    let request = DiscoverPlatform::new("default", "missing");
    let result = discover(&context, &callback, request).await;
    match result {
        Ok(()) => panic!("discovery expected to fail"),
        Err(error) if error.is::<PlatformNotFound>() => (),
        Err(error) => panic!("discovery failed with unexpected error: {:?}", error),
    }
}

#[tokio::test]
async fn platform_not_active() {
    let (callback, _) = fixed_callback().await;
    let context = callback.injector.context.clone();
    let request = DiscoverPlatform::new("default", "test");
    let result = discover(&context, &callback, request).await;
    match result {
        Ok(()) => panic!("discovery expected to fail"),
        Err(error) if error.is::<PlatformNotActive>() => (),
        Err(error) => panic!("discovery failed with unexpected error: {:?}", error),
    }
}

#[tokio::test]
async fn clusters_are_discovered() {
    let (callback, fixture) = fixed_callback().await;
    let context = callback.injector.context.clone();
    let request = DiscoverPlatform::new("default", "unit");
    discover(&context, &callback, request)
        .await
        .expect("platform discovery unsuccessful");

    // Assert the DB includes updated discovery records.
    let injector = callback.injector;
    let cluster1 = injector
        .store
        .query(&context, LookupClusterDiscovery::by("default", "cluster1"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cluster1.nodes[0].node_id, "node1");
    assert_eq!(cluster1.nodes[1].node_id, "node2");
    assert_eq!(cluster1.nodes[2].node_id, "node3");
    let cluster2 = injector
        .store
        .query(&context, LookupClusterDiscovery::by("default", "cluster2"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cluster2.nodes[0].node_id, "node4");

    // Assert the expected events were emitted.
    let timeout = Duration::from_millis(10);
    let mut fixture = fixture;
    let event = fixture.events.pop_change_timeout(timeout).await.unwrap();
    assert_event(&event.code, event.payload);
    let event = fixture.events.pop_change_timeout(timeout).await.unwrap();
    assert_event(&event.code, event.payload);
    let event = fixture.events.pop_change_timeout(timeout).await.unwrap();
    assert_event(&event.code, event.payload);
    let event = fixture.events.pop_change_timeout(timeout).await.unwrap();
    assert_event(&event.code, event.payload);
    let done = fixture.events.pop_change_timeout(timeout).await;
    match done {
        Err(error) if error.is::<tokio::time::error::Elapsed>() => (),
        Err(error) => panic!("unexpected error {:?}", error),
        Ok(event) => panic!("unexpected event {}", event.code),
    };
}

fn assert_event(code: &str, payload: Value) {
    match code {
        crate::events::EVENT_NEW => assert_eq!(
            payload,
            serde_json::json!({
                "ns_id": "default",
                "cluster_id": "cluster1",
                "nodes": [{
                    "agent_address": "unittest://node1",
                    "node_class": "unit",
                    "node_id": "node1",
                    "node_group": null,
                }, {
                    "agent_address": "unittest://node2",
                    "node_class": "unit",
                    "node_id": "node2",
                    "node_group": null,
                }, {
                    "agent_address": "unittest://node3",
                    "node_class": "unit",
                    "node_id": "node3",
                    "node_group": null,
                }]
            })
        ),
        crate::events::EVENT_SYNTHETIC => {
            let cluster_id = payload.get("cluster_id").unwrap().as_str().unwrap();
            assert_eq!(
                payload,
                serde_json::json!({
                    "ns_id": "default",
                    "cluster_id": cluster_id,
                    "active": true,
                    "declaration": {
                        "active": true,
                        "approval": "granted",
                        "definition": null,
                        "expand": {
                            "mode": "Auto",
                            "target_member": null,
                        },
                        "graces": {
                            "expand": 5,
                            "init": 5,
                            "scale_up": 5,
                        },
                        "initialise": {
                            "action_args": null,
                            "mode": "Auto",
                            "search": null,
                        },
                    },
                    "interval": 60,
                    "platform": null,
                })
            );
        }
        crate::events::EVENT_UPDATE => assert_eq!(
            payload,
            serde_json::json!({
                "after": {
                    "ns_id": "default",
                    "cluster_id": "cluster2",
                    "nodes": [{
                        "agent_address": "unittest://node4",
                        "node_class": "unit",
                        "node_id": "node4",
                        "node_group": null,
                    }]
                },
                "before": {
                    "ns_id": "default",
                    "cluster_id": "cluster2",
                    "nodes": [{
                        "agent_address": "unittest://node5",
                        "node_class": "unit",
                        "node_id": "node5",
                        "node_group": null,
                    }]
                }
            })
        ),
        _ => panic!("unexpected event '{}", code),
    }
}
