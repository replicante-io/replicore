//! This package provides interfaces and structs to build <???> agents.
//!
//! The package implements a base `Agent` trait to provide a common interface.
//!
//! To create an agent implement the `Agent` trait for a struct and pass that
//! struct to `AgentRunner::new` to create a runner.
//! The `AgentRunner::run` method will then spin up the API server.
//!
//! # Examples
//!
//! ```
//! extern crate unamed_agent;
//! 
//! use unamed_agent::Agent;
//! use unamed_agent::AgentRunner;
//! use unamed_agent::config::AgentConfig;
//! use unamed_agent::config::AgentWebServerConfig;
//! use unamed_agent::models::AgentVersion;
//! use unamed_agent::models::DatastoreVersion;
//! 
//! 
//! pub struct TestAgent {}
//! 
//! impl TestAgent {
//!     pub fn new() -> TestAgent {
//!         TestAgent {}
//!     }
//! }
//! 
//! impl Agent for TestAgent {
//!     fn datastore_version(&self) -> DatastoreVersion {
//!         DatastoreVersion::new("Test DB", "1.2.3")
//!     }
//! }
//! 
//! 
//! fn main() {
//!     let conf = AgentConfig::new(AgentWebServerConfig::new("127.0.0.1:8080"));
//!     let runner = AgentRunner::new(
//!         Box::new(TestAgent::new()),
//!         conf, AgentVersion::new(
//!             env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
//!             env!("GIT_BUILD_TAINT")
//!         )
//!     );
//!     // This will block the process serving requests.
//!     //runner.run();
//! }
//! ```
extern crate iron;
extern crate iron_json_response;
extern crate router;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate iron_test;


use std::sync::Arc;

use iron::Iron;
use router::Router;

mod api;
pub mod config;
pub mod models;


/// Trait to share common agent code and features.
///
/// Agents should be implemented as structs that implement `BaseAgent`.
pub trait Agent {
    /// Fetch the agent and datastore versions.
    fn datastore_version(&self) -> self::models::DatastoreVersion;
}

/// Container type to hold an Agent trait object.
///
/// This type also adds the Send and Sync requirements needed by the
/// API handlers to hold a reference to an Agent implementation.
type AgentContainer = Arc<Box<Agent + Send + Sync>>;


/// Common implementation for Agents.
///
/// This runner implements common logic that every
/// agent will need on top of the `Agent` trait.
pub struct AgentRunner {
    agent: AgentContainer,
    conf: self::config::AgentConfig,
    version: self::models::AgentVersion
}

impl AgentRunner {
    pub fn new(
        agent: Box<Agent + Send + Sync>,
        conf: self::config::AgentConfig,
        version: self::models::AgentVersion
    ) -> AgentRunner {
        AgentRunner {
            agent: Arc::new(agent),
            conf, version
        }
    }

    /// Starts the Agent process and waits for it to terminate.
    pub fn run(&self) -> () {
        let mut router = Router::new();
        let info = api::InfoHandler::new(
            Arc::clone(&self.agent), self.version.clone()
        );

        router.get("/", api::index, "index");
        router.get("/api/v1/info", info, "info");
        router.get("/api/v1/status", api::status, "status");

        let conf = self.conf.web_server();
        println!("Listening on {} ...", conf.bind_address());
        Iron::new(router)
            .http(conf.bind_address())
            .expect("Unable to start server");
    }
}
