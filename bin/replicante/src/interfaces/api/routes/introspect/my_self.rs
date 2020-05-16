use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Resource;
use actix_web::Responder;

use replicante_service_coordinator::Coordinator;

/// Report information about the node itself.
pub struct MySelf {
    coordinator: Coordinator,
}

impl MySelf {
    pub fn new(coordinator: Coordinator) -> MySelf {
        MySelf { coordinator }
    }

    pub fn resource(&self) -> Resource {
        web::resource("/self")
            .data(self.coordinator.clone())
            .route(web::get().to(responder))
    }
}

async fn responder(coordinator: web::Data<Coordinator>) -> impl Responder {
    let info = coordinator.node_id();
    HttpResponse::Ok().json(info)
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::read_body;
    use actix_web::test::TestRequest;
    use actix_web::App;
    use slog::o;
    use slog::Discard;
    use slog::Logger;

    use replicante_service_coordinator::mock::MockCoordinator;

    use super::MySelf;

    #[actix_rt::test]
    async fn my_self_info() {
        let coordinator = MockCoordinator::new(Logger::root(Discard, o!()));
        let coordinator = coordinator;
        let my_self = MySelf::new(coordinator.mock());
        let app = App::new().service(my_self.resource());
        let mut app = init_service(app).await;

        let req = TestRequest::get().uri("/self").to_request();
        let res = call_service(&mut app, req).await;
        assert!(res.status().is_success());
        let body = read_body(res).await;
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(
            body,
            format!(r#"{{"extra":{{}},"id":"{}"}}"#, coordinator.node_id)
        );
    }
}
