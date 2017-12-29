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
    let agent = MongoDBAgent::new(settings.mongo())
        .expect("Failed to initialise agent");
    let runner = AgentRunner::new(
        Box::new(agent),
        settings.agent(),
        AgentVersion::new(
            env!("GIT_BUILD_HASH"), env!("CARGO_PKG_VERSION"),
            env!("GIT_BUILD_TAINT")
        )
    );
    runner.run();
}
