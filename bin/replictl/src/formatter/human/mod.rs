//! Format output for easy consumption by people interacting with `replictl`.
use time::format_description::BorrowedFormatItem;

use super::ops::Ops;
use super::ops::Responses;
use super::FormatterStrategy;
use crate::globals::Globals;

mod cluster_spec;
mod context;
mod naction;
mod namespace;
mod oaction;
mod platform;

const TIME_FORMAT: &[BorrowedFormatItem<'static>] = time::macros::format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
);

/// Format output for easy consumption by people interacting with `replictl`.
pub struct HumanFormatter;

impl FormatterStrategy for HumanFormatter {
    fn format(&self, _: &Globals, op: Ops) -> Responses {
        match op {
            Ops::ClusterDiscovery(cluster_disc) => {
                self::cluster_spec::discovery(&cluster_disc);
                Responses::Success
            }
            Ops::ClusterSpec(cluster_spec) => match self::cluster_spec::show(&cluster_spec) {
                Err(error) => Responses::Err(error),
                Ok(()) => Responses::Success,
            },
            Ops::ClusterSpecList => {
                Responses::cluster_specs(self::cluster_spec::ClusterSpecList::new())
            }
            Ops::Context(context) => {
                self::context::show(&context);
                Responses::Success
            }
            Ops::ContextList => Responses::contexts(self::context::ContextList::new()),
            Ops::NAction(action) => match self::naction::show(&action) {
                Err(error) => Responses::Err(error),
                Ok(()) => Responses::Success,
            },
            Ops::NActionList => Responses::nactions(self::naction::NActionList::new()),
            Ops::Namespace(namespace) => {
                self::namespace::show(&namespace);
                Responses::Success
            }
            Ops::NamespaceList => Responses::namespaces(self::namespace::NamespaceList::new()),
            Ops::OAction(action) => match self::oaction::show(&action) {
                Err(error) => Responses::Err(error),
                Ok(()) => Responses::Success,
            },
            Ops::OActionList => Responses::oactions(self::oaction::OActionList::new()),
            Ops::OrchestrateReport(report) => {
                match self::cluster_spec::orchestrate_report(&report) {
                    Err(error) => Responses::Err(error),
                    Ok(()) => Responses::Success,
                }
            }
            Ops::Platform(platform) => {
                self::platform::show(&platform);
                Responses::Success
            }
            Ops::PlatformList => Responses::platforms(self::platform::PlatformList::new()),
        }
    }
}
