//! Format output for easy consumption by people interacting with `replictl`.
use super::ops::Ops;
use super::ops::Responses;
use super::FormatterStrategy;
use crate::globals::Globals;

mod cluster_spec;
mod context;
mod namespace;
mod platform;

/// Format output for easy consumption by people interacting with `replictl`.
pub struct HumanFormatter;

impl FormatterStrategy for HumanFormatter {
    fn format(&self, _: &Globals, op: Ops) -> Responses {
        match op {
            Ops::ClusterSpec(cluster_spec) => {
                self::cluster_spec::show(&cluster_spec);
                Responses::Success
            }
            Ops::ClusterSpecList => {
                Responses::cluster_specs(self::cluster_spec::ClusterSpecList::new())
            }
            Ops::Context(context) => {
                self::context::show(&context);
                Responses::Success
            }
            Ops::ContextList => Responses::contexts(self::context::ContextList::new()),
            Ops::Namespace(namespace) => {
                self::namespace::show(&namespace);
                Responses::Success
            }
            Ops::NamespaceList => Responses::namespaces(self::namespace::NamespaceList::new()),
            Ops::Platform(platform) => {
                self::platform::show(&platform);
                Responses::Success
            }
            Ops::PlatformList => Responses::platforms(self::platform::PlatformList::new()),
        }
    }
}
