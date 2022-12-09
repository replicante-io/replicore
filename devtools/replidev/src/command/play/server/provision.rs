use actix_web::post;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use actix_web::Responder;

use replisdk::platform::models::NodeProvisionRequest;
use replisdk::platform::models::NodeProvisionResponse;

use replicante_util_failure::format_fail;

use crate::platform::node_start;
use crate::Conf;

#[post("/provision")]
pub async fn provision(spec: Json<NodeProvisionRequest>, conf: Data<Conf>) -> impl Responder {
    // Check what node to provision.
    let node_group = match spec.cluster.nodes.get(&spec.provision.node_group_id) {
        Some(node_group) => node_group,
        None => {
            let response = serde_json::json!({
                "defined_node_groups": spec.cluster.nodes.keys().collect::<Vec<&String>>(),
                "node_group_id": spec.provision.node_group_id,
                "reason": "provision.node_group_id is not defined in cluster.nodes",
            });
            let response = HttpResponse::BadRequest().json(response);
            return Ok(response);
        }
    };

    // Provision one node at a time.
    let cluster_id = &spec.cluster.cluster_id;
    let node_id = node_start::random_node_id(8);
    let store = &spec.cluster.store;
    let store_version = node_group
        .store_version
        .as_ref()
        .unwrap_or(&spec.cluster.store_version);

    // Prepare the node template environment.
    let paths = crate::settings::paths::PlayPod::new(store, cluster_id, &node_id);
    let variables = match crate::settings::Variables::new(&conf, paths) {
        Ok(variables) => variables,
        Err(error) => {
            let formatted_error = format_fail(&error);
            let response = HttpResponse::InternalServerError().body(formatted_error);
            let error = actix_web::error::InternalError::from_response(error, response);
            return Err(error);
        }
    };

    // TODO: extend variables with (merged) attributes.

    // Create the node pod.
    let node_start_spec = node_start::StartNodeSpec {
        cluster_id,
        node_id: &node_id,
        project: conf.project.to_string(),
        store: &spec.cluster.store,
        store_version: Some(store_version),
        variables,
    };
    if let Err(error) = node_start::start_node(node_start_spec, &conf).await {
        let formatted_error = format_fail(&error);
        let response = HttpResponse::InternalServerError().body(formatted_error);
        let error = actix_web::error::InternalError::from_response(error, response);
        return Err(error);
    };

    // Return the provisioning results.
    let response = NodeProvisionResponse {
        count: 1,
        node_ids: Some(vec![node_id]),
    };
    Ok(HttpResponse::Ok().json(response))
}
