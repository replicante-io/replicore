use anyhow::Context;
use anyhow::Result;
use prettytable::cell;
use prettytable::row;
use slog::Logger;

use crate::context::ContextStore;
use crate::Opt;

/// Execute the command.
pub async fn execute(logger: &Logger, opt: &Opt) -> Result<i32> {
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "ACTIVE",
        "NAME",
        "URL",
        "CA BUNDLE",
        "CLIENT KEY",
        "NAMESPACE",
        "CLUSTER",
        "NODE",
    ]);

    let store = ContextStore::load(logger, opt).await?;
    let active_name = store.active_context_name(opt);
    for (name, context) in store.iter() {
        let ca_bundle = yes_or_no(&context.connection.ca_bundle);
        let client_key = yes_or_no(&context.connection.client_key);
        table.add_row(row![
            if active_name == name { "*" } else { "" },
            name,
            context.connection.url,
            ca_bundle,
            client_key,
            context.scope.namespace.as_deref().unwrap_or("Not set"),
            context.scope.cluster.as_deref().unwrap_or("Not set"),
            context.scope.node.as_deref().unwrap_or("Not set"),
        ]);
    }

    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 2)
        .build();
    table.set_format(format);
    tokio::task::spawn_blocking(move || table.printstd())
        .await
        .with_context(|| "failed to output contexts list")?;
    Ok(0)
}

fn yes_or_no<T>(value: &Option<T>) -> &'static str {
    value.as_ref().map(|_| "Set").unwrap_or("Not set")
}
