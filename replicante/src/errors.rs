use error_chain::ChainedError;

use iron::IronError;
use iron::Response;
use iron::status;
use iron::headers::ContentType;

use serde_json;


error_chain! {
    links {
        Client(::replicante_agent_client::Error, ::replicante_agent_client::ErrorKind);
        Discovery(::replicante_agent_discovery::Error, ::replicante_agent_discovery::ErrorKind);
        Store(::replicante_data_store::Error, ::replicante_data_store::ErrorKind);
        Tracing(::replicante_util_tracing::Error, ::replicante_util_tracing::ErrorKind);
    }

    foreign_links {
        IoError(::std::io::Error);
        YamlDecode(::serde_yaml::Error);
    }
}

impl From<::failure::Error> for Error {
    fn from(error: ::failure::Error) -> Self {
        error.to_string().into()
    }
}

impl From<Error> for IronError {
    fn from(error: Error) -> Self {
        let wrapper = JsonErrorWrapper { error: error.display_chain().to_string() };
        let mut response = Response::with(
            (status::InternalServerError, serde_json::to_string(&wrapper).unwrap())
        );
        response.headers.set(ContentType::json());
        let error = Box::new(error);
        IronError { error, response }
    }
}


#[derive(Serialize)]
struct JsonErrorWrapper {
    error: String,
}


#[cfg(test)]
mod tests {
    use iron::IronResult;
    use iron::Headers;
    use iron::Response;
    use iron::Request;
    use iron::headers::ContentType;

    use iron_test::request;
    use iron_test::response;

    use super::Result;
    use super::ResultExt;

    fn failing(_: &mut Request) -> IronResult<Response> {
        let err: Result<Response> = Err("test".into());
        Ok(err.chain_err(|| "chained").chain_err(|| "failures")?)
    }

    #[test]
    fn error_conversion() {
        let response = request::get("http://host:16016/", Headers::new(), &failing);
        let response = match response {
            Err(error) => error.response,
            Ok(_) => panic!("Request should fail")
        };

        let content_type = response.headers.get::<ContentType>().unwrap().clone();
        assert_eq!(content_type, ContentType::json());

        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, r#"{"error":"Error: failures\nCaused by: chained\nCaused by: test\n"}"#);
    }
}
