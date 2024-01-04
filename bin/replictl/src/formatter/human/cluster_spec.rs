//! Format `Platform` related objects.
use anyhow::Result;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::cluster::ClusterSpec;

/// Format a list of [`ClusterSpecEntry`] objects into a table.
#[derive(Default)]
pub struct ClusterSpecList {
    table: comfy_table::Table,
}

impl ClusterSpecList {
    pub fn new() -> ClusterSpecList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["NAME", "ACTIVE"]);
        ClusterSpecList { table }
    }
}

impl crate::formatter::ClusterSpecList for ClusterSpecList {
    fn append(&mut self, entry: &ClusterSpecEntry) -> Result<()> {
        let active = crate::utils::yes_or_no(entry.active);
        self.table
            .add_row(vec![entry.cluster_id.clone(), active.to_string()]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format a [`ClusterSpec`] for users to inspect.
pub fn show(cluster_spec: &ClusterSpec) {
    let active = crate::utils::yes_or_no(cluster_spec.active);
    println!("Cluster ID: {}", cluster_spec.cluster_id);
    println!("As part of Namespace: {}", cluster_spec.ns_id);
    println!("Active: {}", active);
    println!("Orchestration interval: {} seconds", cluster_spec.interval);
}
