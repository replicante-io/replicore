use serde::Deserialize;
use serde::Serialize;

/// Namespaces are logically isolated groups of resources.
///
/// Namespaces are also a place to define settings such as cluster discovery,
/// agent trasport settings, etcetera.
///
/// Resources includes all concepts in replicante outside users and organisations.
/// This means all clusters, playbooks, roles, and more.
// NOTE: this model jumps the gun a bit as organizations are not a thing yet.
//       This is currently defined to avoid introducing some artifact to pass
//       namespace settings around just to replace it soon after.
//       The current solution is to use the config file and create a "default"
//       namespace model for the code that wants to deal with model and slowly
//       evolve that into a more complex entity at a later date.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Namespace {
    /// Unique ID of the namespace.
    pub ns_id: String,

    /// HTTPS Agent Transport settings.
    #[serde(default)]
    pub https_transport: NsHttpsTransport,
}

impl Namespace {
    /// Return a Namespace object to hardcode in places during gradual implementation of namespaces.
    ///
    /// This allows new code to use namespaces without rewrite of old code or implementation
    /// of the feature itself.
    /// This function will also make it easy to find all the places were namespaces need to be
    /// rolled out thanks to rust's `deprecated` annotation.
    #[allow(non_snake_case)]
    pub fn HARDCODED_FOR_ROLLOUT() -> Namespace {
        Namespace {
            ns_id: "default".to_string(),
            https_transport: NsHttpsTransport {
                ca_bundle: std::env::var("REPLICORE_TMP_DEFAULT_AGENTS_CA_BUNDLE").ok(),
                client_key_id: std::env::var("REPLICORE_TMP_DEFAULT_AGENTS_CLIENT_KEY").ok(),
            },
        }
    }
}

/// HTTPS Agent Transport settings for a namespace.
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NsHttpsTransport {
    /// PEM formatted bundle of CA certificates to validate agent certificates.
    #[serde(default)]
    pub ca_bundle: Option<String>,

    // NOTE: path to a PEM file until a secrets vault is introduced.
    /// Secret ID of a PEM formatted HTTPS client **private** key.
    #[serde(default)]
    pub client_key_id: Option<String>,
}
