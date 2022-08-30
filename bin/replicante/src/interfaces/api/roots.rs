use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;

/// Enumerates all possible API roots.
///
/// All endpoints must fall under one of these roots and are subject to all restrictions
/// of that specific root.
/// The main restriction is that versioned APIs are subject to semver guarantees.
#[allow(clippy::enum_variant_names)]
pub enum APIRoot {
    /// API root for all endpoints that are not yet stable.
    ///
    /// Endpoints in this root are NOT subject to ANY compatibility guarantees!
    UnstableApi,

    /// Replicante catalogues of known kinds and objects.
    UnstableCatalogue,

    /// Replicante core APIs, unstable version.
    UnstableCoreApi,

    /// Instrospection APIs not yet stable.
    UnstableIntrospect,

    /// Specialised endpoints for the WebUI project.
    UnstableWebUI,
}

impl RootDescriptor for APIRoot {
    fn enabled(&self, flags: &APIFlags) -> bool {
        match self {
            APIRoot::UnstableApi
            | APIRoot::UnstableCatalogue
            | APIRoot::UnstableCoreApi
            | APIRoot::UnstableWebUI => match flags.get("unstable") {
                Some(flag) => *flag,
                None => true,
            },
            APIRoot::UnstableIntrospect => match flags.get("unstable") {
                Some(flag) if !flag => *flag,
                _ => match flags.get("introspect") {
                    Some(flag) => *flag,
                    None => true,
                },
            },
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            APIRoot::UnstableApi => "/api/unstable",
            APIRoot::UnstableCatalogue => "/api/unstable/catalogue",
            APIRoot::UnstableCoreApi => "/api/unstable/core",
            APIRoot::UnstableIntrospect => "/api/unstable/introspect",
            APIRoot::UnstableWebUI => "/api/unstable/webui",
        }
    }
}
