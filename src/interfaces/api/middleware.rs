use iron::AfterMiddleware;
use iron::IronError;
use iron::IronResult;
use iron::Request;
use iron::Response;

use slog::Logger;


/// Extracts the request method as a string.
fn request_method(request: &Request) -> String {
    request.method.to_string()
}


/// Extracts the request path as a string.
fn request_path(request: &Request) -> String {
    format!("/{}", request.url.path().join("/"))
}


/// Extracts the response status code as a string.
///
/// # Panics
/// If the response does not have a status set.
fn response_status(response: &Response) -> String {
    response.status.expect("Response instance does not have a status set").to_u16().to_string()
}


/// Iron middleware to log processed requests.
pub struct RequestLogger {
    logger: Logger,
}

impl RequestLogger {
    /// Create a `RequestLogger`.
    pub fn new(logger: Logger) -> RequestLogger {
        RequestLogger { logger }
    }
}

impl AfterMiddleware for RequestLogger {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let method = request_method(req);
        let path = request_path(req);
        let status = response_status(&res);
        info!(
            self.logger, "Request handled";
            "success" => true, "method" => method, "path" => path, "status" => status
        );
        Ok(res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        let method = request_method(req);
        let path = request_path(req);
        let status = response_status(&err.response);
        info!(
            self.logger, "Request failed";
            "success" => false, "method" => method, "path" => path, "status" => status
        );
        Err(err)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use iron::Chain;
    use iron::Headers;
    use iron::IronError;
    use iron::Request;
    use iron::Response;

    use iron::method;
    use iron::status;
    use iron_test::request;

    use slog::Drain;
    use slog::Logger;
    use slog::Never;
    use slog::OwnedKVList;
    use slog::Record;

    use super::RequestLogger;


    #[derive(Debug)]
    struct MockError;
    impl ::std::fmt::Display for MockError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            ::std::fmt::Debug::fmt(self, f)
        }
    }
    impl ::std::error::Error for MockError {
        fn description(&self) -> &str { "MockError" }
    }

    #[derive(Clone)]
    struct MockDrain {
        lines: Arc<Mutex<Vec<String>>>
    }
    impl MockDrain {
        pub fn new(lines: Arc<Mutex<Vec<String>>>) -> MockDrain {
            MockDrain { lines }
        }
    }
    impl Drain for MockDrain {
        type Ok = ();
        type Err = Never;
        fn log(
            &self, record: &Record, _: &OwnedKVList
        ) -> ::std::result::Result<Self::Ok, Self::Err> {
            let line = format!("{}", record.msg());
            self.lines.lock().unwrap().push(line);
            Ok(())
        }
    }

    fn make_chain(lines: Arc<Mutex<Vec<String>>>) -> Chain {
        let drain = MockDrain::new(lines);
        let logger = Logger::root(drain, o!());
        let mut chain = Chain::new(|req: &mut Request| {
            match req.method {
                method::Get => Ok(Response::with((status::Ok, "OK"))),
                _ => {
                    let response = Response::with((status::BadRequest, "Bad"));
                    let err = IronError { response, error: Box::new(MockError) };
                    Err(err)
                }
            }
        });
        chain.link_after(RequestLogger::new(logger));
        chain
    }

    #[test]
    fn request_after() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let chain = make_chain(Arc::clone(&lines));
        request::get("http://host:16016/", Headers::new(), &chain).unwrap();
        let lines = lines.lock().unwrap().clone();
        assert_eq!(lines, vec![String::from("Request handled")]);
    }

    #[test]
    fn request_catch() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let chain = make_chain(Arc::clone(&lines));
        match request::put("http://host:16016/fail", Headers::new(), "", &chain) {
            Ok(_) => panic!("Should have failed"),
            _ => ()
        };
        let lines = lines.lock().unwrap().clone();
        assert_eq!(lines, vec![String::from("Request failed")]);
    }
}
