use iron::prelude::*;
use iron::status;


pub fn status(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "TODO")))
}


#[cfg(test)]
mod tests {
    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    #[test]
    fn status_retruns_todo() {
        let response = request::get(
            "http://localhost:3000/api/v1/status",
            Headers::new(), &super::status
        ).unwrap();
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, "TODO");
    }
}
