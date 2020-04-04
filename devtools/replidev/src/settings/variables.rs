use std::collections::BTreeMap;

use failure::ResultExt;
use handlebars::Handlebars;

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
    vars: BTreeMap<String, String>,
}

impl Variables {
    pub fn new<P: Paths>(conf: &Conf, paths: P) -> Variables {
        let mut vars = BTreeMap::new();
        vars.insert("CONF_ROOT".to_string(), paths.configs().to_string());
        vars.insert("DATA_ROOT".to_string(), paths.data().to_string());
        vars.insert(
            "PKI_ROOT".to_string(),
            Paths::pki(&conf.project).to_string(),
        );
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            vars.insert("PODMAN_HOSTNAME".to_string(), hostname);
        }
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
    pub fn set<S1, S2>(&mut self, name: S1, value: S2) -> &mut Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.vars.insert(name.into(), value.into());
        self
    }
}
