//! Format `OAction` relate objects.
use anyhow::Result;

use replisdk::core::models::api::OActionEntry;
use replisdk::core::models::oaction::OAction;

/// Format a list of [`OActionEntry`] objects into a table.
#[derive(Default)]
pub struct OActionList {
    table: comfy_table::Table,
}

impl OActionList {
    pub fn new() -> OActionList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["ACTION ID", "KIND", "STATUS", "CREATED", "FINISHED"]);
        OActionList { table }
    }
}

impl crate::formatter::OActionList for OActionList {
    fn append(&mut self, entry: &OActionEntry) -> Result<()> {
        let finished = match entry.finished_ts {
            None => String::default(),
            Some(ts) => ts.format(super::TIME_FORMAT)?,
        };
        self.table.add_row(vec![
            entry.action_id.to_string(),
            entry.kind.clone(),
            entry.state.to_string(),
            entry.created_ts.format(super::TIME_FORMAT)?,
            finished,
        ]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format an [`OAction`] for users to inspect.
pub fn show(action: &OAction) -> Result<()> {
    println!("Namespace ID: {}", action.ns_id);
    println!("Cluster ID: {}", action.cluster_id);
    println!("Action ID: {}", action.action_id);

    println!("Kind: {}", action.kind);
    println!("Status: {}", action.state);
    println!(
        "Created at: {}",
        action.created_ts.format(super::TIME_FORMAT)?
    );
    let scheduled = match action.scheduled_ts {
        None => String::from("<Not Scheduled>"),
        Some(ts) => ts.format(super::TIME_FORMAT)?,
    };
    println!("Scheduled at: {}", scheduled);
    let finished = match action.finished_ts {
        None => String::from("<Not Finished>"),
        Some(ts) => ts.format(super::TIME_FORMAT)?,
    };
    println!("Finished at: {}", finished);

    println!("Arguments: {}", serde_json::to_string_pretty(&action.args)?);
    println!(
        "Metadata: {}",
        serde_json::to_string_pretty(&action.metadata)?
    );

    let state = match &action.state_payload {
        None => String::from("<No state saved>"),
        Some(state) => serde_json::to_string_pretty(state)?,
    };
    println!("Saved state: {}", state);
    let error = match &action.state_payload_error {
        None => String::from("<No errors saved>"),
        Some(state) => serde_json::to_string_pretty(state)?,
    };
    println!("Error info: {}", error);
    Ok(())
}
