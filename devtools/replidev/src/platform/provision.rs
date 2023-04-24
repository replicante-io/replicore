use anyhow::Result;
use replisdk::platform::framework::NodeProvisionRequestExt;
use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;
use replisdk::utils::actix::error::Error;
use replisdk_experimental::platform::templates::TemplateContext;

use super::Platform;
use super::PlatformContext;

/// Provision (create) a new node for a cluster.
pub async fn provision(
    platform: &Platform,
    context: &PlatformContext,
    mut request: NodeProvisionRequest,
) -> Result<NodeProvisionResponse> {
    // Check what node to provision.
    let node_group = request.resolve_node_group_remove()?;

    // Provision one node at a time.
    let node_id = super::node_start::random_node_id(8);
    let store_version = node_group
        .store_version
        .unwrap_or(request.cluster.store_version);

    // Prepare the node template environment.
    let mut attributes = request.cluster.attributes.into();
    json_patch::merge(&mut attributes, &node_group.attributes.into());
    let attributes = match attributes {
        serde_json::Value::Object(attributes) => attributes,
        _ => panic!("json_patch::merge of object must return an object"),
    };

    // Lookup and render the template.
    let template_context = TemplateContext {
        attributes,
        cluster_id: request.cluster.cluster_id.clone(),
        store: request.cluster.store.clone(),
        store_version,
    };
    let template = context
        .templates
        .lookup(&template_context)
        .await?
        .ok_or_else(|| {
            let error = anyhow::anyhow!("node template not found");
            Error::with_status(actix_web::http::StatusCode::NOT_FOUND, error)
        })?;
    let pod = template.render(template_context)?;

    // Create the node pod.
    let node_start_spec = super::node_start::StartNodeSpec {
        cluster_id: &request.cluster.cluster_id,
        node_id: &node_id,
        pod,
        project: platform.conf.project.to_string(),
        store: &request.cluster.store,
    };
    super::node_start::start_node(node_start_spec, &platform.conf)
        .await
        .map_err(Error::from)?;

    // Return the provisioning results.
    Ok(NodeProvisionResponse {
        count: 1,
        node_ids: Some(vec![node_id]),
    })
}
