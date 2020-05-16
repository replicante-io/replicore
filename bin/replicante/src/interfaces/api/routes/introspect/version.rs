use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Responder;

use replicante_models_core::api::Version;

lazy_static::lazy_static! {
    static ref REPLICANTE_VERSION: Version = {
        Version {
            commit: String::from(env!("GIT_BUILD_HASH")),
            taint: String::from(env!("GIT_BUILD_TAINT")),
            version: String::from(env!("CARGO_PKG_VERSION")),
        }
    };
}

#[get("/version")]
async fn version() -> impl Responder {
    let version = REPLICANTE_VERSION.clone();
    HttpResponse::Ok().json(version)
}

#[cfg(test)]
mod tests {
    use actix_web::test::init_service;
    use actix_web::test::read_response_json;
    use actix_web::test::TestRequest;
    use actix_web::App;

    use replicante_models_core::api::Version;

    #[actix_rt::test]
    async fn get_version() {
        let app = App::new().service(super::version);
        let mut app = init_service(app).await;

        let req = TestRequest::get().uri("/version").to_request();
        let res: Version = read_response_json(&mut app, req).await;
        assert_eq!(res, *super::REPLICANTE_VERSION);
    }
}
