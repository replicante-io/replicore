use iron::prelude::*;
use iron::status;


pub fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "API endpoints mounted under /api/v1/")))
}


#[cfg(test)]
mod tests {
    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    #[test]
    fn index_points_to_api() {
        let response = request::get(
            "http://localhost:3000/",
            Headers::new(), &super::index
        ).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "API endpoints mounted under /api/v1/");
    }
}
