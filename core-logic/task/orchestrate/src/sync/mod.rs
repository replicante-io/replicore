//! Synchronise information about nodes with the control plane.
//!
//! Synchronisation processes each node individually to refresh the current state.
//! If nodes fail to Synchronise this is noted but the process carries on to prevent
//! individual nodes from blocking all cluster management.
//!
//! The sync process does NOT schedule new node actions, this is expected separately.
use anyhow::Context as AnyContext;
use anyhow::Result;

use replisdk::core::models::node::StoreExtras;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicore_cluster_view::ClusterViewBuilder;
use replicore_context::Context;

mod error;
mod node;
mod store;

use self::error::NodeSpecificCheck;
use self::error::NodeSpecificError;
use crate::init::InitData;

/// Synchronise information about nodes with the control plane.
pub async fn nodes(
    context: &Context,
    data: &InitData,
    new_view: &mut ClusterViewBuilder,
) -> Result<()> {
    // Refresh the state of nodes in the discovery record.
    for node in &data.cluster_current.discovery.nodes {
        let result = sync_node(context, data, new_view, node).await;
        let result = result.with_node_specific()?;
        // TODO: add node/error to orchestrate report.
        println!("~~~ {:?}", result);
    }

    // Delete records about nodes no longer reported.
    // TODO: cancel all actions for deleted nodes.
    // TODO: emit node deleted event.
    // TODO: delete node records (store is responsible for cleaning up across tables/collections).
    Ok(())
}

/// Sync the specified node in isolation.
async fn sync_node(
    context: &Context,
    data: &InitData,
    cluster_new: &mut ClusterViewBuilder,
    node: &ClusterDiscoveryNode,
) -> Result<()> {
    // Create a client to interact with the node.
    let client = data
        .injector
        .clients
        .agent
        .factory(context, &data.cluster_current.spec, node)
        .await?;

    // Fetch essential node information we can't continue without.
    let node_info = self::node::unreachable(&data.cluster_current.spec, node);
    let ag_node = match client.info_node().await.context(NodeSpecificError) {
        Ok(node) => node,
        Err(error) => {
            self::node::persist(context, data, cluster_new, node_info).await?;
            return Err(error);
        }
    };

    // Fetch all other node information and process them as best as possible.
    let store_info = client.info_store().await.context(NodeSpecificError);
    let shards = client.info_shards().await.context(NodeSpecificError);
    let actions_finished = client.actions_finished().await.context(NodeSpecificError);
    let actions_queue = client.actions_queue().await.context(NodeSpecificError);

    // Process fetched information for node sync.
    let incomplete = store_info.is_err()
        || shards.is_err()
        || actions_finished.is_err()
        || actions_queue.is_err();
    let node_info = self::node::process(incomplete, ag_node, node_info);
    self::node::persist(context, data, cluster_new, node_info).await?;

    match store_info {
        Ok(store_info) => {
            let store_info = StoreExtras {
                ns_id: cluster_new.ns_id().to_string(),
                cluster_id: cluster_new.cluster_id().to_string(),
                node_id: node.node_id.clone(),
                attributes: store_info.attributes,
                fresh: true,
            };
            self::store::persist_extras(context, data, cluster_new, node, store_info).await?;
        }
        Err(_error) => {
            // TODO: add to report as event.
            self::store::stale_extras(context, data, cluster_new, node).await?;
        }
    };

    // TODO: persist shards info.
    // TODO: sync finished actions (filter out old actions we would have deleted).
    // TODO: sync actions queue.
    println!("~~~ {:?}", shards);
    println!("~~~ {:?}", actions_finished);
    println!("~~~ {:?}", actions_queue);
    Ok(())
}
