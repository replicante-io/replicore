use anyhow::Context;
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Map;
use serde_json::Value;

use crate::podman::PodPort;
use crate::settings::Paths;
use crate::Conf;

/// Errors related to variables management.
#[derive(Debug, thiserror::Error)]
pub enum VariablesError {
    #[error("could not render variables into string")]
    Render,
}

/// Variables available for substitution in pod definitions.
///
/// Supported variables:
///
///   * `CONF_ROOT`: pod-scoped root to git-committed configs (path to dir).
///   * `DATA_ROOT`: pod-scoped root to git-ignored data (path to dir).
///   * `PODMAN_HOSTNAME`: takes the value of $HOSTNAME where replidev is running.
///
/// Additional custom variables can be added with `Variables::set`.
#[derive(Debug)]
pub struct Variables {
    engine: Handlebars<'static>,
    vars: Map<String, Value>,
}

impl Variables {
    pub fn new<P: Paths>(conf: &Conf, paths: P) -> Variables {
        let mut vars = Map::new();
        vars.insert("CONF_ROOT".to_string(), paths.configs().into());
        vars.insert("DATA_ROOT".to_string(), paths.data().into());
        vars.insert(
            "PKI_ROOT".to_string(),
            <dyn Paths>::pki(&conf.project).into(),
        );
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            vars.insert("PODMAN_HOSTNAME".to_string(), hostname.into());
        }
        vars.insert("extra".to_string(), Map::new().into());
        let engine = Handlebars::new();
        Variables { engine, vars }
    }

    /// Inject supported variables in the value.
    pub fn inject(&self, source: &str) -> Result<String> {
        self.engine
            .render_template(source, &self.vars)
            .context(VariablesError::Render)
            .map_err(anyhow::Error::from)
    }

    /// Add a custom variable with the given value.
    pub fn set<K, V>(&mut self, name: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        let name = name.into();
        if name == "extra" {
            panic!("can't use the reserved variable name 'extra'");
        }
        self.vars.insert(name, value.into());
        self
    }

    /// Add a variable for the POD/NODE name.
    pub fn set_node_name(&mut self, name: String) -> &mut Self {
        self.set("NODE_NAME", name);
        self
    }

    /// Add a variable for each named port.
    ///
    /// The name of the variable is based on the assigned name: HOST_PORT_$NAME.
    /// The value will be the port number on the HOST.
    pub fn set_ports(&mut self, ports: &[PodPort]) -> &mut Self {
        for port in ports {
            if let Some(name) = &port.name {
                let name = name.to_uppercase();
                self.set(format!("HOST_PORT_{}", name), port.host);
            }
        }
        self
    }
}
