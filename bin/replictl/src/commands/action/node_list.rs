use anyhow::Result;
use prettytable::cell;
use prettytable::row;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;
use crate::Opt;

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    // Figure out the cluster being requested.
    let context = ContextStore::active_context(logger, opt).await?;
    let _ns = context.namespace(&opt.context)?;
    let cluster = context.cluster(&opt.context)?;

    // Fetch node actions summaries.
    let client = RepliClient::new(logger, context).await?;
    let actions = client.action_node_summaries(&cluster).await?;

    // Print information in tabular format.
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "CLUSTER ID",
        "NODE ID",
        "ACTION ID",
        "KIND",
        "STATE",
        "CREATED",
        "SCHEDULED",
        "FINISHED",
    ]);

    for action in actions {
        table.add_row(row![
            action.cluster_id,
            action.node_id,
            action.action_id,
            action.kind,
            action.state,
            action.created_ts,
            action
                .scheduled_ts
                .map(|ts| ts.to_string())
                .unwrap_or_else(|| "".into()),
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
