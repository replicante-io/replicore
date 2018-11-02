extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate rdkafka;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;


mod config;
mod error;
mod request;

pub use self::config::Config;
pub use self::error::Result;
pub use self::error::TaskError;
pub use self::request::TaskRequest;
pub use self::request::Tasks;

#[cfg(debug_assertions)]
pub use self::request::MockTasks;


/// Application defined queue definition.
///
/// Letting the application define a type for queues means that application can choose flexibility
/// (the TaskQueue is a String) or compile time checks (the TaskQueue is an enumeration).
///
/// Anything in beetween is also supported with variant attributes and complex structures.
pub trait TaskQueue : Clone + Send + Sync {
    /// Returns the name of the queue tasks should be sent to/received from.
    fn name(&self) -> String;
}
