use actix_web::post;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use actix_web::Responder;

use replisdk::platform::models::NodeDeprovisionRequest;

use replicante_util_failure::format_fail;

use crate::settings::paths::Paths;
use crate::settings::paths::PlayPod;
use crate::Conf;

#[post("/deprovision")]
pub async fn deprovision(
    spec: Json<NodeDeprovisionRequest>,
    conf: Data<Conf>,
) -> impl Responder {
    // Stop the node pod.
    if let Err(error) = crate::podman::pod_stop(&conf, &spec.node_id).await {
        let formatted_error = format_fail(&error);
        let response = HttpResponse::InternalServerError().body(formatted_error);
        let error = actix_web::error::InternalError::from_response(error, response);
        return Err(error);
    }

    // Once the pod node is stopped its data can be deleted.
    let paths = PlayPod::new("<deprovision>", &spec.cluster_id, &spec.node_id);
    let data = paths.data();
    if let Err(error) = crate::podman::unshare(&conf, vec!["rm", "-r", data]).await {
        let formatted_error = format_fail(&error);
        let response = HttpResponse::InternalServerError().body(formatted_error);
        let error = actix_web::error::InternalError::from_response(error, response);
        return Err(error);
    }

    Ok(HttpResponse::NoContent())
}
