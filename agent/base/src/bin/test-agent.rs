extern crate unamed_agent;

use unamed_agent::BaseAgent;
use unamed_agent::VersionInfo;

use unamed_agent::config::AgentConfig;
use unamed_agent::config::AgentWebServerConfig;


pub struct TestAgent {
    conf: AgentConfig,
    version: VersionInfo,
}

impl TestAgent {
    pub fn new(conf: AgentConfig, version: VersionInfo) -> TestAgent {
        TestAgent {
            conf,
            version,
        }
    }
}

impl BaseAgent for TestAgent {
    fn agent_version(&self) -> &VersionInfo {
        &self.version
    }

    fn config(&self) -> &AgentConfig {
        &self.conf
    }
}

fn main() {
    let conf = AgentConfig::new(AgentWebServerConfig::new("127.0.0.1:8080"));
    let version = VersionInfo::new(
        "Test DB", "1.2.3",
        env!("GIT_BUILD_HASH"), "1.2.3", env!("GIT_BUILD_TAINT")
    );

    let agent = TestAgent::new(conf, version);
    agent.run();
}
