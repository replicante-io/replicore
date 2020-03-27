use crate::settings::Paths;

/// Variables available for substitution in pod definitions.
///
/// Supported variables:
///
///   * `{{ CONF_ROOT }}` for a pod-scoped root to git-commited configs (path to dir).
///   * `{{ DATA_ROOT }}` for a pod-scoped root to git-ignored data (path to dir).
///   * `{{ PODMAN_HOSTNAME }}` takes the value of $HOSTNAME where replidev is running.
pub struct Variables<P: Paths> {
    paths: P,
}

impl<P: Paths> Variables<P> {
    pub fn new(paths: P) -> Variables<P> {
        Variables { paths }
    }

    /// Inject supported variables in the value.
    pub fn inject(&self, source: &str) -> String {
        let mut source = source
            .replace("{{ CONF_ROOT }}", &self.paths.configs())
            .replace("{{ DATA_ROOT }}", &self.paths.data());
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            source = source.replace("{{ PODMAN_HOSTNAME }}", &hostname);
        }
        source
    }
}
