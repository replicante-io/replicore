//! Format `Platform` related objects.
use anyhow::Result;

use replisdk::core::models::api::ClusterSpecEntry;
use replisdk::core::models::cluster::ClusterDiscovery;
use replisdk::core::models::cluster::ClusterSpec;

use replicore_cluster_models::OrchestrateReport;

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

/// Format a [`ClusterDiscovery`] for users to inspect.
pub fn discovery(cluster_disc: &ClusterDiscovery) {
    println!("Cluster ID: {}", cluster_disc.cluster_id);
    println!("As part of Namespace: {}", cluster_disc.ns_id);
    println!();

    println!("Discovered cluster nodes:");
    for node in &cluster_disc.nodes {
        println!("  - Node ID: {}", node.node_id);
        println!(
            "    Node group: {}",
            node.node_group.clone().unwrap_or_default()
        );
        println!("    Agent address: {}", node.agent_address);
        println!("    Node class: {}", node.node_class);
    }
}

/// Format an [`OrchestrateReport`] for users to inspect.
pub fn orchestrate_report(report: &OrchestrateReport) -> Result<()> {
    println!("Cluster ID: {}", report.cluster_id);
    println!("As part of Namespace: {}", report.ns_id);
    println!();

    println!("Orchestration mode: {}", report.mode);
    println!(
        "Start time: {}",
        report.start_time.format(super::TIME_FORMAT)?
    );
    println!();

    let mut notes = comfy_table::Table::new();
    notes.set_header(vec!["CATEGORY", "MESSAGE", "DATA"]);
    for note in &report.notes {
        let data = serde_json::to_string(&note.data)?;
        notes.add_row(vec![note.category.to_string(), note.message.clone(), data]);
    }
    println!("The following notes were attached to the report");
    println!("{}", notes);

    Ok(())
}

/// Format a [`ClusterSpec`] for users to inspect.
pub fn show(cluster_spec: &ClusterSpec) -> Result<()> {
    let active = crate::utils::yes_or_no(cluster_spec.active);
    println!("Cluster ID: {}", cluster_spec.cluster_id);
    println!("As part of Namespace: {}", cluster_spec.ns_id);
    println!("Active: {}", active);
    println!("Orchestration interval: {} seconds", cluster_spec.interval);
    println!();

    println!("Cluster declaration details:");
    let declaration = &cluster_spec.declaration;
    let converge = match (declaration.active, declaration.definition.is_some()) {
        (_, false) => "no declaration set",
        (true, _) => "yes",
        (false, _) => "no",
    };
    println!("  Converge to declared cluster: {}", converge);
    println!("  Converge action approval: {}", declaration.approval);
    println!(
        "  Node scale up grace period: {} minutes",
        declaration.grace_up
    );

    if let Some(definition) = &declaration.definition {
        println!();
        println!("  Desired cluster definition:");
        println!("    Cluster store: {}", definition.store);
        println!("    Cluster store version: {}", definition.store_version);

        let attributes = serde_json::to_string_pretty(&definition.attributes)?;
        println!("    Cluster-wide attributes: {}", attributes);

        for (group_id, node_spec) in &definition.nodes {
            println!();
            println!("    Node group {}:", group_id);
            println!("      Node class: {}", node_spec.node_class);
            let store_version = match &node_spec.store_version {
                None => "<Not Set>",
                Some(version) => version,
            };
            println!("      Store version override: {}", store_version);
            println!("      Desired count: {}", node_spec.desired_count);

            let attributes = serde_json::to_string_pretty(&node_spec.attributes)?;
            println!("    Node-specific attributes: {}", attributes);
        }
    }

    Ok(())
}
