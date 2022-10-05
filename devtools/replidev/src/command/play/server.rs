use std::collections::HashMap;

use actix_web::get;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use failure::ResultExt;

use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_util_failure::format_fail;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Actix Web data object attached to the /discover handler.
struct DiscoverData {
    pub agents_address: String,
    pub conf: Conf,
}

impl DiscoverData {
    pub fn from_conf(conf: &Conf) -> DiscoverData {
        let agents_address = conf.resolve_play_server_agents_address();
        let conf = conf.clone();
        DiscoverData {
            agents_address,
            conf,
        }
    }
}

pub async fn run(conf: Conf) -> Result<i32> {
    let bind = conf.play_server_bind.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(DiscoverData::from_conf(&conf)))
            .service(index)
            .service(discover)
    })
    .bind(&bind)
    .with_context(|_| ErrorKind::io("http server failed to bind"))?
    .run();
    println!("--> Server listening at http://{}", bind);
    server
        .await
        .with_context(|_| ErrorKind::io("http server failed to run"))?;
    Ok(0)
}

#[get("/")]
async fn index() -> impl Responder {
    "Server running :-D".to_string()
}

#[get("/discover")]
async fn discover(data: Data<DiscoverData>) -> impl Responder {
    // List all running nodes.
    let nodes = super::node_list::list_nodes(&data.conf).await;
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
    for node in nodes {
        let cluster = node.cluster;
        let address = match node.port_agent {
            Some(port) => format!("https://{}:{}", data.agents_address, port),
            None => continue,
        };
        clusters
            .entry(cluster.clone())
            .and_modify(|cluster| cluster.nodes.push(address.clone()))
            .or_insert_with(|| ClusterDiscovery {
                cluster_id: cluster,
                display_name: None,
                nodes: vec![address],
            });
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
