use anyhow::Result;
use prettytable::row;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;
use crate::Cli;

/// Execute the command.
pub async fn execute(logger: &Logger, cli: &Cli) -> Result<i32> {
    // Fetch requested action summaries.
    let context = ContextStore::active_context(logger, cli).await?;
    let _ns = context.namespace(&cli.context)?;
    let cluster = context.cluster(&cli.context)?;
    let client = RepliClient::new(logger, context).await?;
    let actions = client.action_orchestrator_summaries(&cluster).await?;

    // Print information in tabular format.
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "CLUSTER ID",
        "ACTION ID",
        "KIND",
        "STATE",
        "CREATED",
        "FINISHED",
    ]);

    for action in actions {
        table.add_row(row![
            action.cluster_id,
            action.action_id,
            action.kind,
            action.state,
            action.created_ts,
            action
                .finished_ts
                .map(|ts| ts.to_string())
                .unwrap_or_else(|| "".into()),
        ]);
    }

    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 2)
        .build();
    table.set_format(format);
    table.printstd();
    Ok(0)
}
