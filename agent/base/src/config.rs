/// Stored the base agent configuration options.
///
/// The options are structured in sub-types and this struct
/// acts as a container for all sub-sections.
#[derive(Debug)]
pub struct AgentConfig {
    web_server_conf: AgentWebServerConfig,
}

impl AgentConfig {
    pub fn new(
        web_server_conf: AgentWebServerConfig
    ) -> AgentConfig {
        AgentConfig {
            web_server_conf
        }
    }

    /// Access the web server configuration options.
    pub fn web_server(&self) -> &AgentWebServerConfig {
        &self.web_server_conf
    }
}


/// Store the web server configuration options.
#[derive(Debug)]
pub struct AgentWebServerConfig {
    bind: String,
}

impl AgentWebServerConfig {
    pub fn new(bind: &str) -> AgentWebServerConfig {
        AgentWebServerConfig {
            bind: String::from(bind)
        }
    }

    /// Access the web server bind address.
    pub fn bind_address(&self) -> &str {
        &self.bind
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_web_server_config() {
        let web_conf = AgentWebServerConfig::new("123:456");
        let conf = AgentConfig::new(web_conf);
        assert_eq!("123:456", conf.web_server().bind_address());
    }

    #[test]
    fn bind_address_is_returned() {
        let conf = AgentWebServerConfig::new("123:456");
        assert_eq!("123:456", conf.bind_address());
    }
}
