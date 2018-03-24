//! Module that defines a set of core handlers for the API interface.
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::status;


/// Root index (`/`) handler.
pub fn root_index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Replicante API server")))
}


#[cfg(test)]
mod tests {
    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    use super::root_index;


    #[test]
    fn get_index() {
        let response = request::get("http://host:16016/", Headers::new(), &root_index).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "Replicante API server");
    }
}
