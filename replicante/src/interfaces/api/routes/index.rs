use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::status;


/// Root index (`/`) handler.
pub fn handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Replicante API server")))
}


#[cfg(test)]
mod tests {
    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    use super::handler;


    #[test]
    fn get_index() {
        let response = request::get("http://host:16016/", Headers::new(), &handler).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "Replicante API server");
    }
}
