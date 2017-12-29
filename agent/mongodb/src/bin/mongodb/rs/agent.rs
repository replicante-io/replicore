extern crate unamed_agent;
extern crate unamed_agent_mongodb;

use unamed_agent::AgentRunner;
use unamed_agent::models::AgentVersion;

use unamed_agent_mongodb::MongoDBAgent;
use unamed_agent_mongodb::settings::MongoDBAgentSettings;


fn main() {
    let mut settings = MongoDBAgentSettings::default();
    settings.load(vec![
        "agent-mongodb.yaml",
        "agent-mongodb-rs.yaml"
    ]).expect("Unable to load user settings");

    let runner = AgentRunner::new(
        Box::new(MongoDBAgent::new(settings.mongo())),
        settings.agent(),
        AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
            env!("GIT_BUILD_TAINT")
        )
    );
    runner.run();
}
