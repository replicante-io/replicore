//! Transform manifests to apply before they are sent to servers.
use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::Value;

use crate::context::Context;
use crate::Globals;

/// Determine the manifest's cluster ID attribute based on API version and kind.
static CLUSTER_IDS: Lazy<HashMap<(&str, &str), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(("replicante.io/v0", "clusterspec"), "cluster_id");
    map.insert(("replicante.io/v0", "naction"), "cluster_id");
    map.insert(("replicante.io/v0", "oaction"), "cluster_id");
    map
});

/// Determine the manifest's Namespace ID attribute based on API version and kind.
static NS_IDS: Lazy<HashMap<(&str, &str), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(("replicante.io/v0", "clusterspec"), "ns_id");
    map.insert(("replicante.io/v0", "naction"), "ns_id");
    map.insert(("replicante.io/v0", "namespace"), "id");
    map.insert(("replicante.io/v0", "oaction"), "ns_id");
    map.insert(("replicante.io/v0", "platform"), "ns_id");
    map
});

/// Determine the manifest's Node ID attribute based on API version and kind.
static NODE_IDS: Lazy<HashMap<(&str, &str), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(("replicante.io/v0", "naction"), "node_id");
    map.insert(("replicante.io/v0", "oaction"), "node_id");
    map
});

/// Manifest spec attributes to set/override scope information into.
struct ScopeIDs {
    cluster: Option<String>,
    namespace: Option<String>,
    node: Option<String>,
}

impl ScopeIDs {
    /// Determine the scope attribute names for the current manifest.
    fn ids_for(manifest: &Value) -> Option<ScopeIDs> {
        let api_version = manifest.get("apiVersion").and_then(Value::as_str);
        let kind = manifest.get("kind").and_then(Value::as_str);
        if api_version.is_none() || kind.is_none() {
            return None;
        }

        let api_version = api_version.unwrap();
        let kind = kind.unwrap().to_lowercase();
        let key = (api_version, kind.as_str());

        let ids = ScopeIDs {
            cluster: CLUSTER_IDS.get(&key).map(|cluster| cluster.to_string()),
            namespace: NS_IDS.get(&key).map(|ns| ns.to_string()),
            node: NODE_IDS.get(&key).map(|node| node.to_string()),
        };
        Some(ids)
    }
}

/// Update a manifest with scope IDs loaded form the current context or CLI.
pub fn scope(globals: &Globals, context: &Context, mut manifest: Value) -> Value {
    // Figure out the spec attributes to inject.
    let ids = match ScopeIDs::ids_for(&manifest) {
        Some(ids) => ids,
        None => return manifest,
    };

    // Borrow the spec entry as mutable.
    let spec = manifest
        .get_mut("spec")
        .and_then(|value| value.as_object_mut());
    let spec = match spec {
        Some(spec) => spec,
        None => return manifest,
    };

    // Update the spec attributes with matching scope attributes.
    if let Some(ns) = ids.namespace {
        let scope_ns = context.namespace(&globals.cli.context).ok();
        if let Some(scope_ns) = scope_ns {
            spec.insert(ns, scope_ns.into());
        }
    }
    if let Some(cluster) = ids.cluster {
        let scope_cluster = context.cluster(&globals.cli.context).ok();
        if let Some(scope_cluster) = scope_cluster {
            spec.insert(cluster, scope_cluster.into());
        }
    }
    if let Some(node) = ids.node {
        let scope_node = context.node(&globals.cli.context).ok();
        if let Some(scope_node) = scope_node {
            spec.insert(node, scope_node.into());
        }
    }

    // Done mutating the manifest.
    manifest
}
