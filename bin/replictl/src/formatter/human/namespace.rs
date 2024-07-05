//! Format `Namespace` relate objects.
use anyhow::Result;

use replisdk::core::models::api::NamespaceEntry;
use replisdk::core::models::namespace::Namespace;

/// Format a list of [`NamespaceEntry`] objects into a table.
#[derive(Default)]
pub struct NamespaceList {
    table: comfy_table::Table,
}

impl NamespaceList {
    pub fn new() -> NamespaceList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["NAME", "STATUS"]);
        NamespaceList { table }
    }
}

impl crate::formatter::NamespaceList for NamespaceList {
    fn append(&mut self, entry: &NamespaceEntry) -> Result<()> {
        self.table
            .add_row(vec![entry.id.clone(), entry.status.to_string()]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format a [`Namespace`] for users to inspect.
pub fn show(namespace: &Namespace) {
    let ca_bundle = crate::utils::set_or_not(&namespace.tls.ca_bundle);
    println!("Namespace ID: {}", namespace.id);
    println!("Status: {}", namespace.status);
    println!();

    println!("Default TLS options:");
    println!("  Certificate Authorities Bundle: {}", ca_bundle);
    println!();

    println!("Default settings for cluster orchestration:");
    println!(
        "  Maximum number of node action scheduling attempts: {}",
        namespace.settings.orchestrate.max_naction_schedule_attempts
    );
}
