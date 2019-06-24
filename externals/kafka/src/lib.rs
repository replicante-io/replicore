use std::collections::HashMap;
use std::string::FromUtf8Error;

use failure::Fail;
use prometheus::Registry;
use rdkafka::message::Headers;
use rdkafka::message::OwnedHeaders;
use slog::Logger;

mod config;
mod metrics;
mod stats;

pub use self::config::AckLevel;
pub use self::config::CommonConfig;
pub use self::config::Timeouts;
pub use self::stats::ClientStatsContext;
pub use self::stats::KafkaHealthChecker;

/// Transform a `String -> String` `HashMap` into message headers.
pub fn headers_from_map<S>(map: &HashMap<String, String, S>) -> OwnedHeaders
where
    S: ::std::hash::BuildHasher,
{
    let mut headers = OwnedHeaders::new_with_capacity(map.len());
    for (key, value) in map {
        headers = headers.add(key, value);
    }
    headers
}

/// Transform message headers into a `String -> String` `HashMap`.
pub fn headers_to_map<H>(headers: Option<&H>) -> Result<HashMap<String, String>, InvalidHeaderValue>
where
    H: Headers,
{
    match headers {
        None => Ok(HashMap::new()),
        Some(headers) => {
            let mut map = HashMap::new();
            for idx in 0..headers.count() {
                let (key, value) = headers
                    .get(idx)
                    .expect("should not decode header that does not exist");
                let key = String::from(key);
                let value = match String::from_utf8(value.to_vec()) {
                    Ok(value) => value,
                    Err(error) => return Err(InvalidHeaderValue { error, header: key }),
                };
                map.insert(key, value);
            }
            Ok(map)
        }
    }
}

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::metrics::register_metrics(logger, registry);
}

/// Error returned when UTF8 decoding an header fails.
#[derive(Fail, Debug)]
#[fail(display = "unable to decode value for '{}'", header)]
pub struct InvalidHeaderValue {
    #[cause]
    pub error: FromUtf8Error,
    pub header: String,
}
