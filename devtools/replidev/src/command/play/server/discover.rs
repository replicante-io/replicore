use std::collections::hash_map::Entry;
use std::collections::HashMap;

use actix_web::get;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;
use replisdk::platform::models::ClusterDiscovery;
use replisdk::platform::models::ClusterDiscoveryNode;

use replicante_util_failure::format_fail;

use crate::platform::node_list;
use crate::Conf;

/// Actix Web data object attached to the /discover handler.
pub struct DiscoverData {
    pub agents_address: String,
}

impl DiscoverData {
    pub fn from_conf(conf: &Conf) -> DiscoverData {
        let agents_address = conf.resolve_play_server_agents_address();
        DiscoverData { agents_address }
    }
}

#[get("/discover")]
pub async fn discover(data: Data<DiscoverData>, conf: Data<Conf>) -> impl Responder {
    // List all running nodes.
    let nodes = node_list::list_nodes(&conf).await;
    let nodes = match nodes {
        Ok(nodes) => nodes,
        Err(error) => {
            let formatted_error = format_fail(&error);
            let response = HttpResponse::InternalServerError().body(formatted_error);
            let error = actix_web::error::InternalError::from_response(error, response);
            return Err(error);
        }
    };

    // Format nodes into cluster discover records.
    let mut clusters: HashMap<String, ClusterDiscovery> = HashMap::new();
    for node_pod in nodes {
        let cluster_id = node_pod.cluster;
        let node_id = node_pod.node;
        let agent_address = match node_pod.port_agent {
            Some(port) => format!("https://{}:{}", data.agents_address, port),
            None => continue,
        };
        let node = ClusterDiscoveryNode {
            agent_address,
            node_id,
        };
        match clusters.entry(cluster_id) {
            Entry::Occupied(mut entry) => entry.get_mut().nodes.push(node),
            Entry::Vacant(entry) => {
                let cluster = ClusterDiscovery {
                    cluster_id: entry.key().clone(),
                    nodes: vec![node],
                };
                entry.insert(cluster);
            }
        };
    }
    let clusters: Vec<ClusterDiscovery> =
        clusters.into_iter().map(|(_, cluster)| cluster).collect();

    // Return the result.
    let response = serde_json::json!({
        "clusters": clusters,
        "cursor": null,
    });
    Ok(HttpResponse::Ok().json(response))
}
