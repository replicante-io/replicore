use iron::status;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use lazy_static::lazy_static;

use replicante_data_models::api::Version;

lazy_static! {
    /// Compile-time constant version of core.
    static ref REPLICANTE_VERSION: Version = {
        Version {
            commit: String::from(env!("GIT_BUILD_HASH")),
            taint: String::from(env!("GIT_BUILD_TAINT")),
            version: String::from(env!("CARGO_PKG_VERSION")),
        }
    };
}

/// Version information handler (`/api/v1/version`).
pub fn handler(_: &mut Request) -> IronResult<Response> {
    let version: Version = REPLICANTE_VERSION.clone();
    let mut resp = Response::new();
    resp.set_mut(JsonResponse::json(version))
        .set_mut(status::Ok);
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use iron::Chain;
    use iron::Headers;
    use iron_json_response::JsonResponseMiddleware;
    use iron_test::request;
    use iron_test::response;

    use super::handler;

    #[test]
    fn get_index() {
        let mut chain = Chain::new(&handler);
        chain.link_after(JsonResponseMiddleware::new());
        let response = request::get("http://host:16016/", Headers::new(), &chain).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(
            result_body,
            format!(
                r#"{{"commit":"{}","taint":"{}","version":"{}"}}"#,
                env!("GIT_BUILD_HASH"),
                env!("GIT_BUILD_TAINT"),
                env!("CARGO_PKG_VERSION")
            )
        );
    }
}
