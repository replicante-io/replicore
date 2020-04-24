use std::collections::HashMap;

use actix_web::get;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use failure::ResultExt;

use replicante_models_core::cluster::ClusterDiscovery;
use replicante_util_failure::format_fail;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

// Wrap the IP of the podman host to pass as actix web data.
#[derive(Clone)]
struct PodmanHostIp(String);

pub async fn run(conf: Conf) -> Result<bool> {
    let bind = conf.play_server_bind.clone();
    let ip = PodmanHostIp(conf.podman_host_ip()?);
    let server = HttpServer::new(move || {
        App::new()
            .data(conf.clone())
            .data(ip.clone())
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
    Ok(true)
}

#[get("/")]
async fn index() -> impl Responder {
    "Server running :-D".to_string()
}

#[get("/discover")]
async fn discover(conf: Data<Conf>, ip: Data<PodmanHostIp>) -> impl Responder {
    // List all running nodes.
    let nodes = super::node_list::list_nodes(&conf).await;
    let nodes = match nodes {
        Ok(nodes) => nodes,
        Err(error) => {
            let error = format_fail(&error);
            let response = HttpResponse::InternalServerError().body(error);
            return Err(response);
        }
    };

    // Format nodes into cluster discover records.
    //let mut clusters = HashMap::new();
    let mut clusters: HashMap<String, ClusterDiscovery> = HashMap::new();
    for node in nodes {
        let cluster = node.cluster;
        let address = match node.port_agent {
            Some(port) => format!("http://{}:{}", ip.0, port),
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
