use std::fs::File;

use failure::ResultExt;
use handlebars::Handlebars;
use serde_json::Map;
use serde_json::Value;

use crate::podman::PodPort;
use crate::settings::Paths;
use crate::Conf;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

/// Variables available for substitution in pod definitions.
///
/// Supported variables:
///
///   * `{{ CONF_ROOT }}` for a pod-scoped root to git-commited configs (path to dir).
///   * `{{ DATA_ROOT }}` for a pod-scoped root to git-ignored data (path to dir).
///   * `{{ PODMAN_HOSTNAME }}` takes the value of $HOSTNAME where replidev is running.
///
/// Additional custom variables can be added with `Variables::set`.
pub struct Variables {
    engine: Handlebars<'static>,
    vars: Map<String, Value>,
}

impl Variables {
    pub fn new<P: Paths>(conf: &Conf, paths: P) -> Variables {
        let mut vars = Map::new();
        vars.insert("CONF_ROOT".to_string(), paths.configs().into());
        vars.insert("DATA_ROOT".to_string(), paths.data().into());
        vars.insert("PKI_ROOT".to_string(), Paths::pki(&conf.project).into());
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
            .with_context(|_| ErrorKind::template_render())
            .map_err(Error::from)
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

    /// Add JSON files as extra variables passed to the command line.
    ///
    /// These variables must be provided as string in the form NAME=PATH.
    /// The content of the JSON file is then accessbile as `{{ extra.$NAME }}`.
    pub fn set_cli_var_files(&mut self, files: &[String]) -> Result<&mut Self> {
        for var in files {
            let mut parts = var.splitn(2, '=');
            let name = parts
                .next()
                .expect("splitn must return at least the first item");
            let file = match parts.next() {
                Some(value) => value,
                None => {
                    let error = ErrorKind::invalid_cli_var(name, "unable to extract value");
                    return Err(error.into());
                }
            };
            let data = File::open(file).with_context(|_| ErrorKind::fs_not_allowed(file))?;
            let data = serde_json::from_reader(data)
                .with_context(|_| ErrorKind::invalid_cli_var_file(file))?;
            self.vars
                .get_mut("extra")
                .expect("Variables instance is missing the 'extra' object")
                .as_object_mut()
                .expect("Variables instance has non-object 'extra'")
                .insert(name.to_string(), data);
        }
        Ok(self)
    }

    /// Add extra variables passed to the command line.
    ///
    /// These variables must be provided as string in the form NAME=VALUE.
    /// The `$VALUE` of the variable is then accessbile as `{{ extra.$NAME }}`.
    pub fn set_cli_vars(&mut self, vars: &[String]) -> Result<&mut Self> {
        for var in vars {
            let mut parts = var.splitn(2, '=');
            let name = parts
                .next()
                .expect("splitn must return at least the first item");
            let value = match parts.next() {
                Some(value) => value,
                None => {
                    let error = ErrorKind::invalid_cli_var(name, "unable to extract value");
                    return Err(error.into());
                }
            };
            self.vars
                .get_mut("extra")
                .expect("Variables instance is missing the 'extra' object")
                .as_object_mut()
                .expect("Variables instance has non-object 'extra'")
                .insert(name.to_string(), value.into());
        }
        Ok(self)
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
