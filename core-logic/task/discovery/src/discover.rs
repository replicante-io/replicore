//! Process platform discovery requests.
use anyhow::Result;

use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;

use replicore_context::Context;
use replicore_events::Event;
use replicore_injector::Injector;
use replicore_store::query::LookupClusterSpec;
use replicore_store::query::LookupNamespace;
use replicore_store::query::LookupPlatform;

use crate::callback::Callback;
use crate::errors::NamespaceNotActive;
use crate::errors::NamespaceNotFound;
use crate::errors::PlatformNotActive;
use crate::errors::PlatformNotFound;
use crate::DiscoverPlatform;

/// Process platform discovery requests.
//pub async fn discover(args: CallbackArgs<'_>) -> Result<()> {
pub async fn discover(
    context: &Context,
    callback: &Callback,
    request: DiscoverPlatform,
) -> Result<()> {
    // Lookup the namespace and ensure it is active.
    let op = LookupNamespace::from(request.ns_id.clone());
    let ns = match callback.injector.store.query(context, op).await? {
        Some(ns) => ns,
        None => anyhow::bail!(NamespaceNotFound::new(request.ns_id)),
    };
    if !ns.status.is_active() {
        anyhow::bail!(NamespaceNotActive::new(request.ns_id));
    }

    // Lookup the platform and ensure it is active.
    let op = LookupPlatform::by(&ns.id, &request.name);
    let platform = match callback.injector.store.query(context, op).await? {
        Some(platform) => platform,
        None => anyhow::bail!(PlatformNotFound::new(ns.id, request.name)),
    };
    if !platform.active {
        anyhow::bail!(PlatformNotActive::new(ns.id, request.name));
    }

    // Initialise a platform API client.
    let client = callback.clients.factory(context, &platform).await?;
    let response = client.discover().await?;
    for cluster in response.clusters {
        let cluster = ClusterDiscovery {
            ns_id: platform.ns_id.clone(),
            cluster_id: cluster.cluster_id,
            nodes: cluster.nodes,
        };
        upsert(context, &callback.injector, cluster).await?;
    }
    Ok(())
}

/// Update or insert the cluster discovery record, emitting events as needed.
async fn upsert(context: &Context, injector: &Injector, discovery: ClusterDiscovery) -> Result<()> {
    // If the cluster has no ClusterSpec persist a synthetic one first.
    let spec = LookupClusterSpec::by(&discovery.ns_id, &discovery.cluster_id);
    let spec = injector.store.query(context, spec).await?;
    if spec.is_none() {
        let spec = ClusterSpec::synthetic(&discovery.ns_id, &discovery.cluster_id);
        let event = Event::new_with_payload(crate::events::EVENT_SYNTHETIC, &spec)?;
        injector.events.change(context, event).await?;
        injector.store.persist(context, spec).await?;
    }

    // Lookup any previously stored discovery and emit events.
    let existing = injector.store.query(context, &discovery).await?;
    match existing {
        None => {
            let event = Event::new_with_payload(crate::events::EVENT_NEW, &discovery)?;
            injector.events.change(context, event).await?;
        }
        Some(existing) if existing != discovery => {
            let payload = crate::events::UpdatePayload {
                before: existing,
                after: discovery.clone(),
            };
            let event = Event::new_with_payload(crate::events::EVENT_UPDATE, payload)?;
            injector.events.change(context, event).await?;
        }
        _ => (),
    }

    // Store the new/updated discovery record.
    injector.store.persist(context, discovery).await
}
