//! Module that defines a set of core routes for the API interface.
use prometheus::Registry;
use replicante_util_iron::Router;

use super::APIRoot;

mod index;
mod introspect;

/// Mount core API route handlers.
pub fn mount(router: &mut Router, registry: Registry) {
    // Create the index root for each API root.
    let roots = vec![APIRoot::UnstableAPI];
    for root in roots.into_iter() {
        let mut root = router.for_root(root);
        root.get("/", index::handler, "index");
    }
    self::introspect::mount(router, registry);
}
