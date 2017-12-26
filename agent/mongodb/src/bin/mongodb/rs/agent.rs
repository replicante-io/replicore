extern crate unamed_agent;
extern crate unamed_agent_mongodb;

use unamed_agent::AgentRunner;
use unamed_agent::config::AgentConfig;
use unamed_agent::config::AgentWebServerConfig;
use unamed_agent::models::AgentVersion;

use unamed_agent_mongodb::MongoDBAgent;


fn main() {
    let conf = AgentConfig::new(AgentWebServerConfig::new("127.0.0.1:37017"));
    let runner = AgentRunner::new(
        Box::new(MongoDBAgent::new()),
        conf, AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
            env!("GIT_BUILD_TAINT")
        )
    );
    runner.run();
}
