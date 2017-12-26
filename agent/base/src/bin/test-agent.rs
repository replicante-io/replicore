extern crate unamed_agent;

use unamed_agent::Agent;
use unamed_agent::AgentRunner;
use unamed_agent::AgentVersion;
use unamed_agent::DatastoreVersion;

use unamed_agent::config::AgentConfig;
use unamed_agent::config::AgentWebServerConfig;


pub struct TestAgent {}

impl TestAgent {
    pub fn new() -> TestAgent {
        TestAgent {}
    }
}

impl Agent for TestAgent {
    fn datastore_version(&self) -> DatastoreVersion {
        DatastoreVersion::new("Test DB", "1.2.3")
    }
}


fn main() {
    let conf = AgentConfig::new(AgentWebServerConfig::new("127.0.0.1:8080"));
    let runner = AgentRunner::new(
        Box::new(TestAgent::new()),
        conf, AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
            env!("GIT_BUILD_TAINT")
        )
    );
    runner.run();
}
