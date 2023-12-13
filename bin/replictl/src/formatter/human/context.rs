//! Format `replictl` context objects.
use anyhow::Result;

use crate::context::Context;
use crate::utils::value_or_not_set;

/// Format a list of [`Context`] objects into a table.
#[derive(Default)]
pub struct ContextList {
    table: comfy_table::Table,
}

impl ContextList {
    pub fn new() -> ContextList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "ACTIVE",
            "NAME",
            "URL",
            "CA BUNDLE",
            "CLIENT KEY",
            "NAMESPACE",
            "CLUSTER",
            "NODE",
        ]);
        ContextList { table }
    }
}

impl crate::formatter::ContextList for ContextList {
    fn append(&mut self, name: &str, context: &Context, active: bool) -> Result<()> {
        let ca_bundle = crate::utils::set_or_not(&context.connection.ca_bundle);
        let client_key = crate::utils::set_or_not(&context.connection.client_key);
        self.table.add_row(vec![
            if active { "*" } else { "" },
            name,
            &context.connection.url,
            ca_bundle,
            client_key,
            &value_or_not_set(&context.scope.namespace),
            &value_or_not_set(&context.scope.cluster),
            &value_or_not_set(&context.scope.node),
        ]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format the [`Context`] for users to inspect.
pub fn show(context: &Context) {
    let ca_bundle = crate::utils::set_or_not(&context.connection.ca_bundle);
    let client_key = crate::utils::set_or_not(&context.connection.client_key);
    println!("Control Plane Connection:");
    println!("  URL: {}", context.connection.url);
    println!("  CA Bundle: {}", ca_bundle);
    println!("  Client Key: {}", client_key);
    println!();
    println!("Selected scopes:");
    println!(
        "  Namespace: {}",
        value_or_not_set(&context.scope.namespace)
    );
    println!("  Cluster: {}", value_or_not_set(&context.scope.cluster));
    println!("  Node: {}", value_or_not_set(&context.scope.node));
}
