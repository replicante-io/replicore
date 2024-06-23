//! Steps to load cluster view records from the store.
use anyhow::Result;
use futures_util::stream::TryStreamExt;

use replicore_context::Context;
use replicore_store::query::ListNodes;
use replicore_store::query::ListStoreExtras;
use replicore_store::query::LookupClusterDiscovery;
use replicore_store::query::UnfinishedOAction;
use replicore_store::Store;

use crate::ClusterViewBuilder;

/// Load unfinished OAction records for the cluster.
pub async fn oactions(
    builder: &mut ClusterViewBuilder,
    context: &Context,
    store: &Store,
) -> Result<()> {
    let actions = UnfinishedOAction::for_cluster(builder.ns_id(), builder.cluster_id());
    let mut actions = store.query(context, actions).await?;
    while let Some(action) = actions.try_next().await? {
        builder.oaction(action)?;
    }
    Ok(())
}

/// Load overall cluster information.
pub async fn overall(
    builder: &mut ClusterViewBuilder,
    context: &Context,
    store: &Store,
) -> Result<()> {
    let op = LookupClusterDiscovery::by(builder.ns_id(), builder.cluster_id());
    if let Some(discovery) = store.query(context, op).await? {
        builder.discovery(discovery)?;
    }
    Ok(())
}

/// Load all data about cluster nodes.
pub async fn nodes(
    builder: &mut ClusterViewBuilder,
    context: &Context,
    store: &Store,
) -> Result<()> {
    // Load Node records.
    let op = ListNodes::by(builder.ns_id(), builder.cluster_id());
    let mut nodes = store.query(context, op).await?;
    while let Some(node) = nodes.try_next().await? {
        builder.node_info(node)?;
    }

    // Load StoreExtras records.
    let op = ListStoreExtras::by(builder.ns_id(), builder.cluster_id());
    let mut nodes = store.query(context, op).await?;
    while let Some(node) = nodes.try_next().await? {
        builder.store_extras(node)?;
    }

    Ok(())
}
