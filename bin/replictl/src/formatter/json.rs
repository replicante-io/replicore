//! Format output to JSON.
use anyhow::Result;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::api::PlatformEntry;

use super::ops::Ops;
use super::ops::Responses;
use super::FormatterStrategy;
use crate::context::Context;
use crate::globals::Globals;

/// Format output to JSON.
pub struct JsonFormatter;

impl FormatterStrategy for JsonFormatter {
    fn format(&self, _: &Globals, op: Ops) -> Responses {
        match op {
            Ops::ClusterSpec(cluster_spec) => print_json(cluster_spec),
            Ops::ClusterSpecList => Responses::cluster_specs(ClusterSpecList::default()),
            Ops::Context(context) => print_json(context),
            Ops::ContextList => Responses::contexts(ContextList::default()),
            Ops::Namespace(namespace) => print_json(namespace),
            Ops::NamespaceList => Responses::namespaces(NamespaceList::default()),
            Ops::OAction(action) => print_json(action),
            Ops::OActionList => Responses::oactions(OActionList::default()),
            Ops::Platform(platform) => print_json(platform),
            Ops::PlatformList => Responses::platforms(PlatformList::default()),
        }
    }
}

/// Pretty print a serialisable value as JSON.
fn print_json<V>(value: V) -> Responses
where
    V: serde::Serialize,
{
    let value = match serde_json::to_string_pretty(&value) {
        Ok(value) => value,
        Err(error) => return Responses::Err(anyhow::anyhow!(error)),
    };
    println!("{}", value);
    Responses::Success
}

macro_rules! list_serialiser {
    ($name:ident, $list:ty, $type:ty) => {
        /// Pretty print a list of serialisable items as JSON.
        #[derive(Default)]
        struct $name(Vec<$type>);

        impl $list for $name {
            fn append(&mut self, entry: &$type) -> Result<()> {
                self.0.push(entry.clone());
                Ok(())
            }

            fn finish(&mut self) -> Result<()> {
                let value = serde_json::to_string_pretty(&self.0)?;
                println!("{}", value);
                Ok(())
            }
        }
    };
}

list_serialiser!(ClusterSpecList, crate::formatter::ClusterSpecList, ClusterSpecEntry);
list_serialiser!(NamespaceList, crate::formatter::NamespaceList, NamespaceEntry);
list_serialiser!(OActionList, crate::formatter::OActionList, OActionEntry);
list_serialiser!(PlatformList, crate::formatter::PlatformList, PlatformEntry);

/// Pretty print an list of context information.
#[derive(Default)]
struct ContextList(Vec<ContextInfo>);

impl crate::formatter::ContextList for ContextList {
    fn append(&mut self, name: &str, entry: &Context, active: bool) -> Result<()> {
        self.0.push(ContextInfo {
            name: name.to_string(),
            active,
            context: entry.clone(),
        });
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        let value = serde_json::to_string_pretty(&self.0)?;
        println!("{}", value);
        Ok(())
    }
}

/// Container for context entries to list.
#[derive(serde::Serialize)]
struct ContextInfo {
    name: String,
    active: bool,
    context: Context,
}
