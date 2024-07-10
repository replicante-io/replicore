//! Format `NAction` relate objects.
use anyhow::Result;

use replisdk::core::models::api::NActionEntry;
use replisdk::core::models::naction::NAction;

/// Format a list of [`NActionEntry`] objects into a table.
#[derive(Default)]
pub struct NActionList {
    table: comfy_table::Table,
}

impl NActionList {
    pub fn new() -> NActionList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "NODE ID",
            "ACTION ID",
            "KIND",
            "STATUS",
            "CREATED",
            "FINISHED",
        ]);
        NActionList { table }
    }
}

impl crate::formatter::NActionList for NActionList {
    fn append(&mut self, entry: &NActionEntry) -> Result<()> {
        let finished = match entry.finished_time {
            None => String::default(),
            Some(ts) => ts.format(super::TIME_FORMAT)?,
        };
        self.table.add_row(vec![
            entry.node_id.to_string(),
            entry.action_id.to_string(),
            entry.kind.clone(),
            entry.state.to_string(),
            entry.created_time.format(super::TIME_FORMAT)?,
            finished,
        ]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format a [`NAction`] for users to inspect.
pub fn show(action: &NAction) -> Result<()> {
    println!("Namespace ID: {}", action.ns_id);
    println!("Cluster ID: {}", action.cluster_id);
    println!("Node ID: {}", action.node_id);
    println!("Action ID: {}", action.action_id);

    println!("Kind: {}", action.kind);
    println!("Phase: {}", action.state.phase);
    println!(
        "Created at: {}",
        action.created_time.format(super::TIME_FORMAT)?
    );
    let scheduled = match action.scheduled_time {
        None => String::from("<Not Scheduled>"),
        Some(ts) => ts.format(super::TIME_FORMAT)?,
    };
    println!("Scheduled at: {}", scheduled);
    let finished = match action.finished_time {
        None => String::from("<Not Finished>"),
        Some(ts) => ts.format(super::TIME_FORMAT)?,
    };
    println!("Finished at: {}", finished);
    println!();

    println!("Arguments: {}", serde_json::to_string_pretty(&action.args)?);
    println!(
        "Metadata: {}",
        serde_json::to_string_pretty(&action.metadata)?
    );

    let state = match &action.state.payload {
        None => String::from("<No state saved>"),
        Some(state) => serde_json::to_string_pretty(state)?,
    };
    println!("Saved state: {}", state);
    let error = match &action.state.error {
        None => String::from("<No errors saved>"),
        Some(state) => serde_json::to_string_pretty(state)?,
    };
    println!("Error info: {}", error);
    Ok(())
}
