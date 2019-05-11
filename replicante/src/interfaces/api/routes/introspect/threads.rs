use iron::status;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;

use humthreads::registered_threads;
use humthreads::ThreadStatus;

/// Threads introspection handler (`/introspect/threads`).
pub fn handler(_: &mut Request) -> IronResult<Response> {
    let mut threads = registered_threads();
    threads.sort_unstable_by_key(|t| t.name.clone());
    let threads = ThreadsResponse::new(threads);
    let mut resp = Response::new();
    resp.set_mut(JsonResponse::json(threads))
        .set_mut(status::Ok);
    Ok(resp)
}

/// Wrap the `humthreads::registered_threads` list to expose as structured data.
#[derive(Serialize)]
struct ThreadsResponse {
    threads: Vec<ThreadStatus>,
    warning: &'static [&'static str],
}

impl ThreadsResponse {
    fn new(threads: Vec<ThreadStatus>) -> ThreadsResponse {
        let warning = &[
            "This list is NOT provided from an OS-layer instrumentation.",
            "As such, some threads may not be reported in this list.",
        ];
        ThreadsResponse { threads, warning }
    }
}
