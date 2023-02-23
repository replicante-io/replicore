use anyhow::Result;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;
use replisdk::utils::actix::error::Error;

use super::Platform;

/// Provision (create) a new node for a cluster.
pub async fn provision(
    platform: &Platform,
    mut request: NodeProvisionRequest,
) -> Result<NodeProvisionResponse> {
    // Check what node to provision.
    let node_group = match request.cluster.nodes.remove(&request.provision.node_group_id) {
        Some(node_group) => node_group,
        None => {
            let error = anyhow::anyhow!("provision.node_group_id is not defined in cluster.nodes");
            let response = serde_json::json!({
                "defined_node_groups": request.cluster.nodes.keys().collect::<Vec<&String>>(),
                "error_msg": error.to_string(),
                "node_group_id": request.provision.node_group_id,
            });
            let error = Error::from(error).use_strategy(response);
            anyhow::bail!(error);
        }
    };

    // Provision one node at a time.
    let cluster_id = &request.cluster.cluster_id;
    let node_id = super::node_start::random_node_id(8);
    let store = &request.cluster.store;
    let store_version = node_group
        .store_version
        .unwrap_or(request.cluster.store_version);

    // Prepare the node template environment.
    let mut attributes = request.cluster.attributes.into();
    json_patch::merge(&mut attributes, &node_group.attributes.into());

    let paths = crate::settings::paths::PlayPod::new(store, cluster_id, &node_id);
    let variables = crate::settings::Variables::new(&platform.conf, paths);

    // Create the node pod.
    let node_start_spec = super::node_start::StartNodeSpec {
        cluster_id,
        node_id: &node_id,
        project: platform.conf.project.to_string(),
        store: &request.cluster.store,
        store_version: Some(&store_version),
        attributes,
        variables,
    };
    super::node_start::start_node(node_start_spec, &platform.conf)
        .await
        .map_err(replisdk::utils::actix::error::Error::from)?;

    // Return the provisioning results.
    Ok(NodeProvisionResponse {
        count: 1,
        node_ids: Some(vec![node_id]),
    })
}