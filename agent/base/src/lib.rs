//! This package provides interfaces and structs to build <???> agents.
//!
//! The package implements a base agent trait to provide common logic
//! shared across all agent implementation.
//!
//! To create an agent implement the BaseAgent trait for a struct and
//! then call the run method on the trait:
//!
//!
//! # Examples
//!
//! ```
//! use unamed_agent::BaseAgent;
//! use unamed_agent::VersionInfo;
//!
//! use unamed_agent::config::AgentConfig;
//! use unamed_agent::config::AgentWebServerConfig;
//!
//! pub struct MyAgent {
//!     conf: AgentConfig,
//!     version: VersionInfo
//! }
//!
//! impl MyAgent {
//!     pub fn new(conf: AgentConfig, version: VersionInfo) -> MyAgent {
//!         MyAgent {
//!             conf,
//!             version
//!         }
//!     }
//! }
//!
//! impl BaseAgent for MyAgent {
//!     fn agent_version(&self) -> &VersionInfo {
//!         &self.version
//!     }
//!
//!     fn config(&self) -> &AgentConfig {
//!         &self.conf
//!     }
//! }
//!
//! let conf = AgentConfig::new(AgentWebServerConfig::new("127.0.0.1:8080"));
//! let version = VersionInfo::new(
//!     "Test DB", "1.2.3",
//!     "dcd2b81a60262a78960b4b97ccdc2d6dfd12ac5b", ".1.2.3", "not tainted"
//! );
//!
//! let agent = MyAgent::new(conf, version);
//! // Running the agent is a blocking operation so it is commented out.
//! //agent.run();
//! ```
extern crate iron;
extern crate iron_json_response;
extern crate router;

extern crate serde;
extern crate serde_json;

#[macro_use] extern crate serde_derive;

#[cfg(test)] extern crate iron_test;


use iron::Iron;
use router::Router;


pub mod config;
mod api;


/// Trait to share common agent code and features.
///
/// Agents should be implemented as structs that implement `BaseAgent`.
pub trait BaseAgent {
    /// Fetch the agent and datastore versions.
    fn agent_version(&self) -> &VersionInfo;

    /// Fetch the agent configuration.
    fn config(&self) -> &config::AgentConfig;

    /// Starts the Agent process and waits for it to terminate.
    fn run(&self) -> () {
        let mut router = Router::new();
        let info = api::InfoHandler::new(self.agent_version().clone());

        router.get("/", api::index, "index");
        router.get("/api/v1/info", info, "info");
        router.get("/api/v1/status", api::status, "status");

        let conf = self.config().web_server();
        println!("Listening on {} ...", conf.bind_address());
        Iron::new(router)
            .http(conf.bind_address())
            .expect("Unable to start server");
    }
}


/// Stores agent and datastore versions.
#[derive(Clone, Debug, Serialize)]
pub struct VersionInfo {
    datastore: DatastoreVersion,
    version: AgentVersion,
}

#[derive(Clone, Debug, Serialize)]
struct AgentVersion {
    checkout: String,
    number: String,
    taint: String,
}

#[derive(Clone, Debug, Serialize)]
struct DatastoreVersion {
    name: String,
    version: String,
}

impl VersionInfo {
    pub fn new(
        datastore_name: &str, datastore_version: &str,
        agent_checkout: &str, agent_number: &str, agent_taint: &str
    ) -> VersionInfo {
        VersionInfo {
            datastore: DatastoreVersion {
                name: String::from(datastore_name),
                version: String::from(datastore_version)
            },
            version: AgentVersion {
                checkout: String::from(agent_checkout),
                number: String::from(agent_number),
                taint: String::from(agent_taint)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::VersionInfo;

    #[test]
    fn version_info_serialises_to_json() {
        let version = VersionInfo::new(
            "DB", "1.2.3",
            "dcd", "1.2.3", "tainted"
        );
        let payload = serde_json::to_string(&version).unwrap();
        let expected = r#"{"datastore":{"name":"DB","version":"1.2.3"},"version":{"checkout":"dcd","number":"1.2.3","taint":"tainted"}}"#;
        assert_eq!(payload, expected);
    }
}
