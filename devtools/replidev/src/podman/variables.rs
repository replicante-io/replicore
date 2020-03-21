/// Variables available for substitution in pod definitions.
///
/// Supported variables:
///
///   * `{{ CONF_ROOT }}` for a pod-scoped root to git-commited configs (path to dir).
///   * `{{ DATA_ROOT }}` for a pod-scoped root to git-ignored data (path to dir).
///   * `{{ PODMAN_HOSTNAME }}` takes the value of $HOSTNAME where replidev is running.
pub struct Variables {
    conf_root: String,
    data_root: String,
}

impl Variables {
    pub fn new<S>(pod: S) -> Variables
    where
        S: std::fmt::Display,
    {
        Variables {
            conf_root: format!("./devtools/configs/{}", pod),
            data_root: format!("./devtools/data/{}", pod),
        }
    }

    /// Inject supported variables in the value.
    pub fn inject(&self, source: &str) -> String {
        let mut source = source
            .replace("{{ CONF_ROOT }}", &self.conf_root)
            .replace("{{ DATA_ROOT }}", &self.data_root);
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            source = source.replace("{{ PODMAN_HOSTNAME }}", &hostname);
        }
        source
    }
}
