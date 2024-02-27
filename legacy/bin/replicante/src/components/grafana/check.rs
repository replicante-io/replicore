use actix_web::get;
use actix_web::Responder;

#[get("/")]
pub async fn check() -> impl Responder {
    "Grafana SimpleJson Annotations API endpoints".to_string()
}
