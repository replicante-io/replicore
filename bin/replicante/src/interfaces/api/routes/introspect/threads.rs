use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde_derive::Serialize;

use humthreads::registered_threads;
use humthreads::ThreadStatus;

#[get("/threads")]
async fn threads() -> impl Responder {
    let mut list = registered_threads();
    list.sort_unstable_by_key(|t| t.name.clone());
    let list = ThreadsResponse::new(list);
    HttpResponse::Ok().json(list)
}

/// Wrap the `humthreads::registered_threads` list to expose as structured data.
#[derive(Serialize)]
struct ThreadsResponse {
    threads: Vec<ThreadStatus>,
    warning: &'static [&'static str],
}

impl ThreadsResponse {
    fn new(list: Vec<ThreadStatus>) -> ThreadsResponse {
        let warning = &[
            "This list is NOT provided from an OS-layer instrumentation.",
            "As such, some threads may not be reported in this list.",
        ];
        ThreadsResponse {
            threads: list,
            warning,
        }
    }
}
